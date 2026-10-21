//! Local-only enrollment and import planning. Never include secret input in diagnostics.
use crate::otp::{
    from_otp_uri::FromOtpUri,
    otp_element::{OTPDatabase, OTPElement},
    otp_type::OTPType,
};
use clap::ValueEnum;
use color_eyre::eyre::{Result, eyre};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum DuplicatePolicy {
    #[default]
    Skip,
    Replace,
    KeepBoth,
}

#[derive(Default)]
pub struct ImportReport {
    pub elements: Vec<OTPElement>,
    pub errors: Vec<String>,
}

impl ImportReport {
    pub fn record(&mut self, index: usize, result: Result<Vec<OTPElement>>) {
        match result {
            Ok(elements) => {
                for (entry, mut element) in elements.into_iter().enumerate() {
                    match validate(&mut element) {
                        Ok(()) => self.elements.push(element),
                        Err(error) => self
                            .errors
                            .push(format!("Entry {index}.{}: {error}", entry + 1)),
                    }
                }
            }
            // Deserializers may echo invalid input (including secrets). Keep diagnostics redacted.
            Err(_) => self.errors.push(format!(
                "Entry {index}: invalid or unsupported account data"
            )),
        }
    }
}

pub fn validate(element: &mut OTPElement) -> Result<()> {
    element.secret = element.secret.trim().replace(' ', "");
    if element.type_ == OTPType::Motp {
        element.secret.make_ascii_lowercase();
        hex::decode(&element.secret).map_err(|_| eyre!("invalid hex secret"))?;
    } else {
        element.secret.make_ascii_uppercase();
        element.secret = element.secret.trim_end_matches('=').to_owned();
        data_encoding::BASE32_NOPAD
            .decode(element.secret.as_bytes())
            .map_err(|_| eyre!("invalid base32 secret"))?;
    }
    if element.secret.is_empty() {
        return Err(eyre!("empty secret"));
    }
    if !(1..=10).contains(&element.digits) {
        return Err(eyre!("digits must be between 1 and 10"));
    }
    if element.period == 0 {
        return Err(eyre!("period must be positive"));
    }
    if element.type_ == OTPType::Hotp && element.counter.is_none() {
        return Err(eyre!("HOTP counter is required"));
    }
    if matches!(element.type_, OTPType::Motp | OTPType::Yandex) && element.pin.is_none() {
        return Err(eyre!("PIN is required"));
    }
    Ok(())
}

pub fn parse_uris(content: &str) -> ImportReport {
    let mut report = ImportReport::default();
    for (index, uri) in content.split_whitespace().enumerate() {
        let result = if uri.starts_with("otpauth-migration://") {
            super::google_authenticator::parse_migration_uri(uri)
        } else if uri.starts_with("otpauth://") {
            OTPElement::from_otp_uri(uri).map(|e| vec![e])
        } else {
            Err(eyre!("unsupported URI"))
        };
        report.record(index + 1, result);
    }
    report
}

pub fn decode_qr(bytes: &[u8]) -> Result<ImportReport> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16384);
    limits.max_image_height = Some(16384);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let gray = reader.decode()?.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare_from_greyscale(
        gray.width() as usize,
        gray.height() as usize,
        |x, y| gray.get_pixel(x as u32, y as u32).0[0],
    );
    let grids = prepared.detect_grids();
    if grids.is_empty() {
        return Err(eyre!("No QR code detected in image"));
    }
    let mut report = ImportReport::default();
    for (index, grid) in grids.into_iter().enumerate() {
        match grid.decode() {
            Ok((_, text)) => {
                let decoded = parse_uris(&text);
                report.elements.extend(decoded.elements);
                report.errors.extend(
                    decoded
                        .errors
                        .into_iter()
                        .map(|e| format!("QR {}: {e}", index + 1)),
                );
            }
            Err(_) => report
                .errors
                .push(format!("QR {}: could not decode QR data", index + 1)),
        }
    }
    Ok(report)
}

/// Decode container entries independently, preserving source entry numbers on errors.
pub fn parse_json(value: Value, format: &str) -> Result<ImportReport> {
    let mut report = ImportReport::default();
    let (pointer, wrapper): (&str, fn(Value) -> Value) = match format {
        "cotp" => (
            "/elements",
            |v| serde_json::json!({"version":2,"elements":[v]}),
        ),
        "andotp" | "converted" | "authy-exported" => ("", |v| serde_json::json!([v])),
        "aegis" => ("/db/entries", |v| serde_json::json!({"db":{"entries":[v]}})),
        "freeotp-plus" => (
            "/tokens",
            |v| serde_json::json!({"tokenOrder":[],"tokens":[v]}),
        ),
        "otp-uri" => ("/items", |v| serde_json::json!({"items":[v]})),
        _ => return Err(eyre!("Unsupported import format")),
    };
    let entries = value
        .pointer(pointer)
        .and_then(Value::as_array)
        .ok_or_else(|| eyre!("Import file does not contain the expected account list"))?;
    for (index, value) in entries.iter().enumerate() {
        let value = wrapper(value.clone());
        let result = match format {
            "cotp" => convert::<OTPDatabase>(value),
            "andotp" => convert::<Vec<OTPElement>>(value),
            "converted" => convert::<super::converted::ConvertedJsonList>(value),
            "aegis" => convert::<super::aegis::AegisJson>(value),
            "freeotp-plus" => convert::<super::freeotp_plus::FreeOTPPlusJson>(value),
            "authy-exported" => convert::<super::authy_remote_debug::AuthyExportedList>(value),
            "otp-uri" => convert::<crate::exporters::otp_uri::OtpUriList>(value),
            _ => unreachable!(),
        };
        report.record(index + 1, result);
    }
    Ok(report)
}

pub fn convert<T>(value: Value) -> Result<Vec<OTPElement>>
where
    T: serde::de::DeserializeOwned + TryInto<Vec<OTPElement>>,
    T::Error: std::fmt::Debug,
{
    serde_json::from_value::<T>(value)?
        .try_into()
        .map_err(|_| eyre!("Invalid account data"))
}

pub fn detect_format(value: &Value) -> Result<&'static str> {
    if value.get("elements").is_some() {
        Ok("cotp")
    } else if value.pointer("/db/entries").is_some() {
        Ok("aegis")
    } else if value.get("db").is_some_and(Value::is_string) {
        Ok("aegis-encrypted")
    } else if value.get("tokens").is_some() {
        Ok("freeotp-plus")
    } else if value.get("items").is_some() {
        Ok("otp-uri")
    } else if let Some(first) = value.as_array().and_then(|a| a.first()) {
        if first.get("period").is_some() {
            Ok("andotp")
        } else if first.get("algorithm").is_some() {
            Ok("converted")
        } else if first.get("uri").is_some() {
            Ok("authy-exported")
        } else {
            Err(eyre!(
                "Cannot detect JSON format; specify an import format flag"
            ))
        }
    } else {
        Err(eyre!(
            "Cannot detect JSON format; specify an import format flag"
        ))
    }
}

fn same_account(a: &OTPElement, b: &OTPElement) -> bool {
    a.secret
        .trim_end_matches('=')
        .eq_ignore_ascii_case(b.secret.trim_end_matches('='))
        && a.type_ == b.type_
        && a.algorithm == b.algorithm
        && a.digits == b.digits
        && a.period == b.period
        && a.pin == b.pin
}

pub fn apply(
    database: &mut OTPDatabase,
    elements: Vec<OTPElement>,
    policy: DuplicatePolicy,
) -> Result<(usize, usize, usize)> {
    let (mut added, mut replaced, mut skipped) = (0, 0, 0);
    for mut element in elements {
        let duplicate = database
            .elements_ref()
            .iter()
            .position(|old| same_account(old, &element));
        match (duplicate, policy) {
            (Some(_), DuplicatePolicy::Skip) => skipped += 1,
            (Some(index), DuplicatePolicy::Replace) => {
                let old = database.mut_element(index).unwrap();
                // Importing an older backup must never roll an HOTP counter backwards.
                if element.type_ == OTPType::Hotp {
                    element.counter = old.counter.max(element.counter);
                }
                element.id = old.id.clone();
                element.alias = old.alias.clone();
                element.tags = old.tags.clone();
                element.favorite = old.favorite;
                element.order = old.order;
                *old = element;
                database.mark_modified();
                replaced += 1;
            }
            _ => {
                database.add_element(element)?;
                added += 1;
            }
        }
    }
    Ok((added, replaced, skipped))
}

#[cfg(test)]
mod tests {
    use super::*;
    const URI: &str = "otpauth://totp/Example:alice?secret=JBSWY3DP";
    #[test]
    fn detects_and_decodes_every_qr_in_screenshot() {
        let qr = qrcode::QrCode::new(URI)
            .unwrap()
            .render::<image::Luma<u8>>()
            .min_dimensions(256, 256)
            .build();
        let mut screenshot =
            image::GrayImage::from_pixel(qr.width() * 2 + 40, qr.height(), image::Luma([255]));
        image::imageops::overlay(&mut screenshot, &qr, 0, 0);
        image::imageops::overlay(&mut screenshot, &qr, i64::from(qr.width() + 40), 0);
        let mut bytes = std::io::Cursor::new(Vec::new());
        screenshot
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let report = decode_qr(bytes.get_ref()).unwrap();
        assert!(report.errors.is_empty());
        assert_eq!(report.elements.len(), 2);
    }
    #[test]
    fn repeated_import_is_idempotent_and_hotp_never_rolls_back() {
        let mut database = OTPDatabase::default();
        let uri = "otpauth://hotp/Example:alice?secret=JBSWY3DP&counter=99";
        assert_eq!(
            apply(
                &mut database,
                parse_uris(uri).elements,
                DuplicatePolicy::Skip
            )
            .unwrap(),
            (1, 0, 0)
        );
        assert_eq!(
            apply(
                &mut database,
                parse_uris(uri).elements,
                DuplicatePolicy::Skip
            )
            .unwrap(),
            (0, 0, 1)
        );
        let old_uri = uri.replace("99", "5");
        assert_eq!(
            apply(
                &mut database,
                parse_uris(&old_uri).elements,
                DuplicatePolicy::Replace
            )
            .unwrap(),
            (0, 1, 0)
        );
        assert_eq!(database.elements_ref()[0].counter, Some(99));
        assert_eq!(
            apply(
                &mut database,
                parse_uris(uri).elements,
                DuplicatePolicy::KeepBoth
            )
            .unwrap(),
            (1, 0, 0)
        );
    }
    #[test]
    fn invalid_entries_are_reported_without_exposing_input() {
        let report = parse_uris(&format!("{URI}\notpauth://totp/hidden?secret=TOPSECRET"));
        assert_eq!(report.elements.len(), 1);
        assert_eq!(report.errors.len(), 1);
        assert!(!report.errors[0].contains("TOPSECRET"));
    }
    #[test]
    fn json_entries_fail_independently() {
        let report = parse_json(serde_json::json!({"items":[URI,42]}), "otp-uri").unwrap();
        assert_eq!(report.elements.len(), 1);
        assert_eq!(report.errors.len(), 1);
    }
    #[test]
    fn freeotp_hotp_counter_survives_import() {
        let report = parse_json(serde_json::json!({"tokens":[{"type":"HOTP","algo":"SHA1","counter":42,"digits":6,"issuerExt":"Example","label":"alice","period":30,"secret":[72,101,108,108,111]}]}),"freeotp-plus").unwrap();
        assert!(report.errors.is_empty());
        assert_eq!(report.elements[0].counter, Some(42));
    }
}
