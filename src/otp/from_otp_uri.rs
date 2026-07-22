use eyre::ErrReport;
use url::Url;

use super::{
    otp_algorithm::OTPAlgorithm,
    otp_element::{OTPElement, OTPElementBuilder},
    otp_type::OTPType,
};

pub trait FromOtpUri: Sized {
    fn from_otp_uri(otp_uri: &str) -> eyre::Result<Self>;
}

impl FromOtpUri for OTPElement {
    fn from_otp_uri(otp_uri: &str) -> eyre::Result<Self> {
        // Parse the raw URI: percent-decoding must only ever happen on the
        // individual components. Decoding the whole URI up front turns encoded
        // structural characters into real ones (e.g. "%23" -> "#" makes the
        // rest of the URI a fragment, "%26" -> "&" splits a query value) and
        // decodes every query value twice, corrupting values that contain a
        // literal "%25".
        let parsed_uri = Url::parse(otp_uri).map_err(ErrReport::from)?;

        let otp_type = parsed_uri
            .host_str()
            .map_or_else(|| "TOTP".to_string(), str::to_uppercase);

        let (issuer, label) = get_issuer_and_label(&parsed_uri)?;

        // The secret is taken as-is: case normalization is applied by
        // OTPElementBuilder depending on the OTP type. Base32 secrets
        // (TOTP/HOTP/Steam/Yandex) are uppercased, while MOTP secrets are hex
        // strings fed as text into MD5, so their case must not be folded to
        // uppercase or the generated codes would be wrong.
        let secret = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "secret")
            .map(|(_k, v)| v.to_string())
            .ok_or(ErrReport::msg("Secret not found in OTP Uri"))?;

        let algorithm = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "algorithm")
            .map_or_else(|| "SHA1".to_string(), |(_k, v)| v.to_uppercase());

        let digits = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "digits")
            .map_or(6, |(_k, v)| v.parse::<u64>().unwrap_or(6));

        let period = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "period")
            .map_or(30, |(_k, v)| v.parse::<u64>().unwrap_or(30));

        let counter = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "counter")
            .and_then(|(_k, v)| v.parse::<u64>().ok());

        let pin = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "pin")
            .map(|(_k, v)| v.to_string());

        // Build through OTPElementBuilder so its validation (secret encoding,
        // period, digits) applies to URI imports too. The type must be set
        // after the secret, so the builder can normalize the secret case.
        OTPElementBuilder::default()
            .secret(secret)
            .type_(OTPType::from(otp_type.as_str()))
            .issuer(issuer)
            .label(label)
            .digits(digits)
            .algorithm(OTPAlgorithm::from(algorithm.as_str()))
            .period(period)
            .counter(counter)
            .pin(pin)
            .build()
    }
}

/// Extracts the "issuer:label" parts from the first path segment of the URI.
///
/// The raw segment is percent-decoded first, then split on ':'. Decoding
/// before splitting keeps the historical behavior of treating an encoded
/// colon ("%3A") as the issuer/label separator (see GH issue 548).
fn issuer_label_segments(parsed_uri: &Url) -> eyre::Result<Vec<String>> {
    let raw_segment = parsed_uri
        .path_segments()
        .ok_or(ErrReport::msg("Failed to collect path segments"))?
        .next()
        .ok_or(ErrReport::msg("No path segments found"))?;

    let decoded = urlencoding::decode(raw_segment)
        .map_err(ErrReport::from)?
        .into_owned();

    Ok(decoded
        .split(':')
        .map(std::borrow::ToOwned::to_owned)
        .collect())
}

fn get_issuer_and_label(parsed_uri: &Url) -> eyre::Result<(String, String)> {
    // Find the first path segments, OTP Uris should not have others
    let first_segment = issuer_label_segments(parsed_uri)?;

    let first = first_segment.first().cloned();
    let second = first_segment.get(1).cloned();

    match (first, second) {
        (Some(i), Some(l)) => Ok((i, l)),
        (Some(l), None) => {
            let issuer = parsed_uri
                .query_pairs()
                .find(|(k, _v)| k == "issuer")
                .map(|(_k, v)| v.to_string())
                .unwrap_or_default();
            Ok((issuer, l))
        }
        _ => Err(ErrReport::msg("No label found in OTP uri")),
    }
}

#[cfg(test)]
mod tests {
    use super::FromOtpUri;
    use crate::otp::otp_element::OTPElement;

    #[test]
    fn test_encoded_hash_in_label_does_not_break_query_parsing() {
        // "%23" must stay part of the label; pre-decoding the whole URI turned
        // it into "#", making everything after it a fragment and losing the
        // secret.
        let uri = "otpauth://totp/C%23Corp?secret=JBSWY3DPEHPK3PXP";

        let element = OTPElement::from_otp_uri(uri).unwrap();

        assert_eq!("C#Corp", element.label);
        assert_eq!("JBSWY3DPEHPK3PXP", element.secret);
    }

    #[test]
    fn test_encoded_question_mark_in_label_does_not_break_query_parsing() {
        let uri = "otpauth://totp/Que%3FStion?secret=JBSWY3DPEHPK3PXP";

        let element = OTPElement::from_otp_uri(uri).unwrap();

        assert_eq!("Que?Stion", element.label);
        assert_eq!("JBSWY3DPEHPK3PXP", element.secret);
    }

    #[test]
    fn test_encoded_ampersand_in_label_and_query_value() {
        let uri = "otpauth://totp/A%26B?secret=JBSWY3DPEHPK3PXP&issuer=C%26D";

        let element = OTPElement::from_otp_uri(uri).unwrap();

        assert_eq!("A&B", element.label);
        assert_eq!("C&D", element.issuer);
        assert_eq!("JBSWY3DPEHPK3PXP", element.secret);
    }

    #[test]
    fn test_percent_encoded_label_is_decoded_exactly_once() {
        // Label text "50%off" is encoded as "50%25off" and must not be
        // decoded twice.
        let uri = "otpauth://totp/50%25off?secret=JBSWY3DPEHPK3PXP";

        let element = OTPElement::from_otp_uri(uri).unwrap();

        assert_eq!("50%off", element.label);
    }

    #[test]
    fn test_double_encoded_label_keeps_literal_percent_sequence() {
        // Label text literally containing "%25" is encoded as "%2525" and
        // must decode back to "%25", not to "%".
        let uri = "otpauth://totp/x%2525y?secret=JBSWY3DPEHPK3PXP";

        let element = OTPElement::from_otp_uri(uri).unwrap();

        assert_eq!("x%25y", element.label);
    }
}
