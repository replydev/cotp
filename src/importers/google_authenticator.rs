//! Importer for Google Authenticator's "Export accounts" feature.
//!
//! Google Authenticator does not produce a file backup anymore: exporting
//! accounts generates one or more QR codes encoding
//! `otpauth-migration://offline?data=<base64>` URIs. The `data` payload is a
//! base64-encoded protobuf message (`MigrationPayload`) holding the exported
//! OTP parameters.
//!
//! This module accepts a text file containing one or more of those URIs (one
//! per line, as produced by scanning each exported QR code) and turns them
//! into [`OTPElement`]s. The protobuf payload is decoded with `prost` using
//! the hand-declared message definitions below (Google's schema is stable and
//! tiny, so no `.proto`/`protoc` build step is needed).

use std::{fs::read_to_string, path::PathBuf};

use base64::{Engine as _, engine::general_purpose};
use data_encoding::BASE32_NOPAD;
use eyre::{Result, eyre};
use prost::Message;
use url::Url;

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

const MIGRATION_SCHEME: &str = "otpauth-migration";

/// Google's `MigrationPayload` protobuf. Only the `otp_parameters` field is
/// relevant to us; the batch/version fields are ignored (prost skips fields we
/// do not declare).
#[derive(Clone, PartialEq, Message)]
struct MigrationPayload {
    #[prost(message, repeated, tag = "1")]
    otp_parameters: Vec<OtpParameters>,
}

/// A single exported OTP entry. `algorithm`, `digits` and `otp_type` are
/// protobuf enums, decoded here as their underlying `i32` values.
#[derive(Clone, PartialEq, Message)]
struct OtpParameters {
    #[prost(bytes = "vec", tag = "1")]
    secret: Vec<u8>,
    #[prost(string, tag = "2")]
    name: String,
    #[prost(string, tag = "3")]
    issuer: String,
    #[prost(int32, tag = "4")]
    algorithm: i32,
    #[prost(int32, tag = "5")]
    digits: i32,
    #[prost(int32, tag = "6")]
    otp_type: i32,
    #[prost(int64, tag = "7")]
    counter: i64,
}

// Enum values as defined by Google Authenticator.
const ALGORITHM_SHA256: i32 = 2;
const ALGORITHM_SHA512: i32 = 3;
const ALGORITHM_MD5: i32 = 4;
const DIGIT_COUNT_EIGHT: i32 = 2;
const OTP_TYPE_HOTP: i32 = 1;

/// Reads a file containing one or more `otpauth-migration://` URIs and decodes
/// every OTP parameter contained in them.
pub fn import_from_google_authenticator(path: PathBuf) -> Result<Vec<OTPElement>> {
    let content = read_to_string(path)?;
    import_from_string(&content)
}

/// Parses every `otpauth-migration://` URI found in `content` (tokens are
/// split on whitespace, so both single- and multi-line inputs work).
fn import_from_string(content: &str) -> Result<Vec<OTPElement>> {
    let uris: Vec<&str> = content
        .split_whitespace()
        .filter(|token| token.starts_with(MIGRATION_SCHEME))
        .collect();

    if uris.is_empty() {
        return Err(eyre!(
            "No otpauth-migration:// URI found. Export your accounts from Google Authenticator, \
             scan the generated QR code(s) with any scanner and save the resulting \
             otpauth-migration:// URI(s) into the file you import."
        ));
    }

    let mut elements = Vec::new();
    for uri in uris {
        elements.extend(parse_migration_uri(uri)?);
    }
    Ok(elements)
}

/// Decodes a single `otpauth-migration://offline?data=...` URI.
fn parse_migration_uri(uri: &str) -> Result<Vec<OTPElement>> {
    let parsed = Url::parse(uri)?;

    if parsed.scheme() != MIGRATION_SCHEME {
        return Err(eyre!("Unexpected URI scheme: {}", parsed.scheme()));
    }

    let data = parsed
        .query_pairs()
        .find(|(k, _)| k == "data")
        .map(|(_, v)| v.into_owned())
        .ok_or_else(|| eyre!("Missing 'data' parameter in otpauth-migration URI"))?;

    // The url crate already percent-decodes the value, so we can decode the
    // raw base64 (standard alphabet, with padding) directly.
    let decoded = general_purpose::STANDARD
        .decode(data.as_bytes())
        .map_err(|e| eyre!("Invalid base64 in otpauth-migration data: {e}"))?;

    let payload = MigrationPayload::decode(decoded.as_slice())
        .map_err(|e| eyre!("Invalid otpauth-migration protobuf payload: {e}"))?;

    payload
        .otp_parameters
        .into_iter()
        .map(into_otp_element)
        .collect()
}

fn into_otp_element(params: OtpParameters) -> Result<OTPElement> {
    if params.secret.is_empty() {
        return Err(eyre!("Encountered an OTP entry with an empty secret"));
    }

    let algorithm = match params.algorithm {
        ALGORITHM_SHA256 => OTPAlgorithm::Sha256,
        ALGORITHM_SHA512 => OTPAlgorithm::Sha512,
        ALGORITHM_MD5 => OTPAlgorithm::Md5,
        // 0 (unspecified) and 1 (SHA1) both map to SHA1.
        _ => OTPAlgorithm::Sha1,
    };

    let digits = if params.digits == DIGIT_COUNT_EIGHT {
        8
    } else {
        6
    };

    let type_ = if params.otp_type == OTP_TYPE_HOTP {
        OTPType::Hotp
    } else {
        OTPType::Totp
    };

    let counter = (type_ == OTPType::Hotp).then_some(params.counter.max(0) as u64);

    Ok(OTPElement {
        secret: BASE32_NOPAD.encode(&params.secret),
        issuer: params.issuer,
        label: params.name,
        digits,
        type_,
        algorithm,
        period: 30,
        counter,
        pin: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wraps the given `OtpParameters` into a `MigrationPayload` and builds an
    /// `otpauth-migration://` URI around its prost-encoded bytes, so the tests
    /// exercise the same wire format the importer decodes.
    fn build_uri(entries: Vec<OtpParameters>) -> String {
        let payload = MigrationPayload {
            otp_parameters: entries,
        }
        .encode_to_vec();
        let data = general_purpose::STANDARD.encode(&payload);
        let encoded = urlencoding::encode(&data).into_owned();
        format!("otpauth-migration://offline?data={encoded}")
    }

    fn otp_parameters(
        secret: &[u8],
        name: &str,
        issuer: &str,
        algorithm: i32,
        digits: i32,
        otp_type: i32,
        counter: i64,
    ) -> OtpParameters {
        OtpParameters {
            secret: secret.to_vec(),
            name: name.to_string(),
            issuer: issuer.to_string(),
            algorithm,
            digits,
            otp_type,
            counter,
        }
    }

    #[test]
    fn parses_single_totp_entry() {
        // BASE32_NOPAD("Hello") == "JBSWY3DP"
        let uri = build_uri(vec![otp_parameters(
            b"Hello",
            "alice@google.com",
            "Example",
            1,
            1,
            2,
            0,
        )]);

        let elements = import_from_string(&uri).unwrap();

        assert_eq!(
            vec![OTPElement {
                secret: "JBSWY3DP".to_string(),
                issuer: "Example".to_string(),
                label: "alice@google.com".to_string(),
                digits: 6,
                type_: OTPType::Totp,
                algorithm: OTPAlgorithm::Sha1,
                period: 30,
                counter: None,
                pin: None,
            }],
            elements
        );
    }

    #[test]
    fn parses_hotp_and_algorithm_and_digits() {
        // algorithm 3 = SHA512, digits 2 = EIGHT, type 1 = HOTP, counter 42
        let uri = build_uri(vec![otp_parameters(b"Hello", "bob", "Acme", 3, 2, 1, 42)]);

        let elements = import_from_string(&uri).unwrap();

        assert_eq!(
            vec![OTPElement {
                secret: "JBSWY3DP".to_string(),
                issuer: "Acme".to_string(),
                label: "bob".to_string(),
                digits: 8,
                type_: OTPType::Hotp,
                algorithm: OTPAlgorithm::Sha512,
                period: 30,
                counter: Some(42),
                pin: None,
            }],
            elements
        );
    }

    #[test]
    fn parses_multiple_entries_across_multiple_uris() {
        let uri1 = build_uri(vec![otp_parameters(b"Hello", "a", "IssuerA", 1, 1, 2, 0)]);
        let uri2 = build_uri(vec![otp_parameters(b"Hello", "b", "IssuerB", 2, 1, 2, 0)]);
        let content = format!("{uri1}\n{uri2}\n");

        let elements = import_from_string(&content).unwrap();

        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0].issuer, "IssuerA");
        assert_eq!(elements[0].algorithm, OTPAlgorithm::Sha1);
        assert_eq!(elements[1].issuer, "IssuerB");
        assert_eq!(elements[1].algorithm, OTPAlgorithm::Sha256);
    }

    #[test]
    fn decodes_multiple_entries_in_a_single_uri() {
        let uri = build_uri(vec![
            otp_parameters(b"Hello", "a", "IssuerA", 1, 1, 2, 0),
            otp_parameters(b"Hello", "b", "IssuerB", 1, 1, 1, 7),
        ]);

        let elements = import_from_string(&uri).unwrap();

        assert_eq!(elements.len(), 2);
        assert_eq!(elements[1].counter, Some(7));
    }

    #[test]
    fn errors_when_no_migration_uri_present() {
        let err = import_from_string("otpauth://totp/foo?secret=JBSWY3DP").unwrap_err();
        assert!(err.to_string().contains("No otpauth-migration"));
    }

    #[test]
    fn errors_on_invalid_base64() {
        let err = import_from_string("otpauth-migration://offline?data=not*base64").unwrap_err();
        assert!(err.to_string().contains("Invalid base64"));
    }

    #[test]
    fn errors_on_invalid_protobuf() {
        // Valid base64 but not a valid protobuf message.
        let data = general_purpose::STANDARD.encode([0xff, 0xff, 0xff, 0xff]);
        let encoded = urlencoding::encode(&data).into_owned();
        let uri = format!("otpauth-migration://offline?data={encoded}");
        let err = import_from_string(&uri).unwrap_err();
        assert!(err.to_string().contains("protobuf"));
    }
}
