//! Native plaintext migration formats. See docs/imports.md for primary schemas.
use super::enrollment::ImportReport;
use crate::otp::{
    from_otp_uri::FromOtpUri,
    otp_element::{OTPElement, OTPElementBuilder},
    otp_type::OTPType,
};
use color_eyre::eyre::{Result, eyre};
use serde_json::Value;
use std::io::{Cursor, Read};

pub fn onepassword_archive(bytes: &[u8]) -> Result<ImportReport> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| eyre!("Invalid 1PUX archive"))?;
    let mut entry = archive
        .by_name("export.data")
        .map_err(|_| eyre!("1PUX archive is missing export.data"))?;
    if entry.size() > 32 * 1024 * 1024 {
        return Err(eyre!("1PUX export.data exceeds 32 MiB"));
    }
    let mut json = zeroize::Zeroizing::new(Vec::new());
    entry
        .by_ref()
        .take(32 * 1024 * 1024 + 1)
        .read_to_end(&mut json)?;
    if json.len() > 32 * 1024 * 1024 {
        return Err(eyre!("1PUX export.data exceeds 32 MiB"));
    }
    let value =
        serde_json::from_slice(&json).map_err(|_| eyre!("Invalid 1PUX export.data JSON"))?;
    parse(&value, "onepassword")
}

fn array<'a>(value: &'a Value, field: &str) -> Result<&'a [Value]> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| eyre!("Missing account container: {field}"))
}
fn text<'a>(value: &'a Value, pointer: &str) -> &'a str {
    value.pointer(pointer).and_then(Value::as_str).unwrap_or("")
}

pub fn parse(value: &Value, format: &str) -> Result<ImportReport> {
    let mut report = ImportReport::default();
    match format {
        "twofas" => {
            if value
                .get("servicesEncrypted")
                .is_some_and(|v| !v.is_null() && v.as_str() != Some(""))
            {
                return Err(eyre!(
                    "Encrypted 2FAS backups are not supported; export an unencrypted .2fas file"
                ));
            }
            if value
                .get("schemaVersion")
                .and_then(Value::as_u64)
                .is_some_and(|v| v > 4)
            {
                return Err(eyre!("Unsupported 2FAS backup schema version"));
            }
            for (index, service) in array(value, "services")?.iter().enumerate() {
                report.record(
                    index + 1,
                    twofas(service).map(|mut e| {
                        if let Some(group) = value
                            .get("groups")
                            .and_then(Value::as_array)
                            .into_iter()
                            .flatten()
                            .find(|g| g.get("id") == service.get("groupId"))
                            && let Some(name) = group.get("name").and_then(Value::as_str)
                        {
                            e.tags.push(name.to_owned());
                        }
                        vec![e]
                    }),
                );
            }
        }
        "bitwarden" => {
            if value.get("encrypted").and_then(Value::as_bool) == Some(true) {
                return Err(eyre!(
                    "Encrypted Bitwarden exports are not supported; use an unencrypted JSON export"
                ));
            }
            for (index, item) in array(value, "items")?.iter().enumerate() {
                let Some(totp) = item.pointer("/login/totp").filter(|v| !v.is_null()) else {
                    continue;
                };
                if totp.as_str() == Some("") {
                    continue;
                }
                let result = totp
                    .as_str()
                    .ok_or_else(|| eyre!("Invalid TOTP value"))
                    .and_then(|secret| {
                        from_secret_or_uri(
                            secret,
                            text(item, "/name"),
                            text(item, "/login/username"),
                        )
                    })
                    .map(|mut e| {
                        e.favorite = item
                            .get("favorite")
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        vec![e]
                    });
                report.record(index + 1, result);
            }
        }
        "onepassword" => {
            let mut index = 0;
            for account in array(value, "accounts")? {
                for vault in array(account, "vaults")? {
                    for item in array(vault, "items")? {
                        for section in item
                            .pointer("/details/sections")
                            .and_then(Value::as_array)
                            .into_iter()
                            .flatten()
                        {
                            for field in section
                                .get("fields")
                                .and_then(Value::as_array)
                                .into_iter()
                                .flatten()
                            {
                                let Some(totp) =
                                    field.pointer("/value/totp").filter(|v| !v.is_null())
                                else {
                                    continue;
                                };
                                if totp.as_str() == Some("") {
                                    continue;
                                }
                                index += 1;
                                let result = totp
                                    .as_str()
                                    .ok_or_else(|| eyre!("Invalid TOTP value"))
                                    .and_then(|secret| {
                                        from_secret_or_uri(
                                            secret,
                                            text(item, "/overview/title"),
                                            text(item, "/overview/subtitle"),
                                        )
                                    })
                                    .map(|mut e| {
                                        e.tags = item
                                            .pointer("/overview/tags")
                                            .and_then(Value::as_array)
                                            .into_iter()
                                            .flatten()
                                            .filter_map(Value::as_str)
                                            .map(str::to_owned)
                                            .collect();
                                        e.favorite = item
                                            .get("favIndex")
                                            .and_then(Value::as_i64)
                                            .is_some_and(|v| v > 0);
                                        vec![e]
                                    });
                                report.record(index, result);
                            }
                        }
                    }
                }
            }
        }
        _ => return Err(eyre!("Unknown migration format")),
    }
    Ok(report)
}

fn from_secret_or_uri(secret: &str, issuer: &str, label: &str) -> Result<OTPElement> {
    if secret.starts_with("otpauth://") {
        let mut element = OTPElement::from_otp_uri(secret)?;
        if element.issuer.is_empty() {
            element.issuer = issuer.to_owned();
        }
        if element.label.is_empty() {
            element.label = label.to_owned();
        }
        return Ok(element);
    }
    if let Some(secret) = secret.strip_prefix("steam://") {
        return OTPElementBuilder::default()
            .secret(secret)
            .issuer(issuer)
            .label(label)
            .type_(OTPType::Steam)
            .digits(5u64)
            .build();
    }
    OTPElementBuilder::default()
        .secret(secret)
        .issuer(issuer)
        .label(label)
        .build()
}

fn twofas(service: &Value) -> Result<OTPElement> {
    let otp = service
        .get("otp")
        .ok_or_else(|| eyre!("Missing OTP parameters"))?;
    let secret = service
        .get("secret")
        .and_then(Value::as_str)
        .ok_or_else(|| eyre!("Missing OTP secret"))?;
    let kind = otp
        .get("tokenType")
        .and_then(Value::as_str)
        .unwrap_or("TOTP")
        .to_ascii_lowercase();
    let issuer = otp
        .get("issuer")
        .and_then(Value::as_str)
        .unwrap_or(text(service, "/name"));
    let label = otp
        .get("account")
        .and_then(Value::as_str)
        .or_else(|| otp.get("label").and_then(Value::as_str))
        .unwrap_or(text(service, "/name"));
    // A URI funnels all format validation through the same strict path as QR enrollment.
    let mut uri = url::Url::parse(&format!("otpauth://{kind}/"))?;
    uri.set_path(&format!("{issuer}:{label}"));
    {
        let mut query = uri.query_pairs_mut();
        query.append_pair("secret", secret);
        for (key, default) in [("digits", "6"), ("period", "30"), ("algorithm", "SHA1")] {
            match otp.get(key).filter(|v| !v.is_null()) {
                Some(Value::String(value)) => {
                    query.append_pair(key, value);
                }
                Some(Value::Number(value)) => {
                    query.append_pair(key, &value.to_string());
                }
                None => {
                    query.append_pair(key, default);
                }
                _ => return Err(eyre!("Invalid OTP parameter")),
            }
        }
        if let Some(counter) = otp.get("counter").filter(|v| !v.is_null()) {
            query.append_pair("counter", &counter.to_string());
        }
    }
    let mut element = OTPElement::from_otp_uri(uri.as_str())?;
    element.order = service.pointer("/order/position").and_then(Value::as_i64);
    Ok(element)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::otp_algorithm::OTPAlgorithm;
    use serde_json::json;
    const TWOFAS: &str = include_str!("../../test_samples/migrations/twofas.2fas");
    const BITWARDEN: &str = include_str!("../../test_samples/migrations/bitwarden.json");
    const ONEPASSWORD: &str = include_str!("../../test_samples/migrations/onepassword.data");

    #[test]
    fn official_schema_fixtures_preserve_parameters_and_metadata() {
        let report = parse(&serde_json::from_str(TWOFAS).unwrap(), "twofas").unwrap();
        assert!(report.errors.is_empty());
        let e = &report.elements[0];
        assert_eq!(e.type_, OTPType::Hotp);
        assert_eq!(e.counter, Some(42));
        assert_eq!(e.digits, 8);
        assert_eq!(e.algorithm, OTPAlgorithm::Sha256);
        assert_eq!(e.tags, ["work"]);
        assert_eq!(e.order, Some(2));
        let report = parse(&serde_json::from_str(BITWARDEN).unwrap(), "bitwarden").unwrap();
        assert!(report.errors.is_empty());
        assert_eq!(report.elements.len(), 2);
        assert!(report.elements[0].favorite);
        assert_eq!(report.elements[0].period, 60);
        assert_eq!(report.elements[1].label, "bob");
        let report = parse(&serde_json::from_str(ONEPASSWORD).unwrap(), "onepassword").unwrap();
        assert!(report.errors.is_empty());
        assert_eq!(report.elements.len(), 1);
        assert_eq!(report.elements[0].algorithm, OTPAlgorithm::Sha512);
        assert_eq!(report.elements[0].tags, ["work"]);
        assert!(report.elements[0].favorite);
        let report = crate::importers::enrollment::parse_uris(include_str!(
            "../../test_samples/migrations/ente.txt"
        ));
        assert!(report.errors.is_empty());
        assert_eq!(report.elements.len(), 2);
        assert_eq!(report.elements[1].counter, Some(42));
    }

    #[test]
    fn onepassword_zip_reads_export_data_without_extracting_attachments() {
        use std::io::Write;
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("export.data", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(ONEPASSWORD.as_bytes()).unwrap();
        writer
            .start_file("files/attachment", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"not an OTP").unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let report = onepassword_archive(&bytes).unwrap();
        assert!(report.errors.is_empty());
        assert_eq!(report.elements.len(), 1);
        assert!(onepassword_archive(b"not a ZIP").is_err());
    }

    #[test]
    fn unsupported_encryption_is_rejected_and_errors_redact_secrets() {
        assert!(
            parse(
                &json!({"servicesEncrypted":"ciphertext","services":[]}),
                "twofas"
            )
            .is_err()
        );
        assert!(parse(&json!({"encrypted":true,"items":[]}), "bitwarden").is_err());
        assert!(parse(&json!({"schemaVersion":99,"services":[]}), "twofas").is_err());
        let report = parse(&json!({"items":[{"login":{"totp":"TOP_SECRET_INVALID"}},{"login":{"totp":"JBSWY3DP"}}]}), "bitwarden").unwrap();
        assert_eq!(report.errors.len(), 1);
        assert!(!report.errors[0].contains("TOP_SECRET"));
        assert_eq!(report.elements.len(), 1);
    }
}
