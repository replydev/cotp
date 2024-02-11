use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Serialize};

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

#[derive(Serialize, Deserialize)]
pub struct FreeOTPPlusJson {
    #[serde(rename = "tokenOrder")]
    token_order: Vec<String>,
    tokens: Vec<FreeOTPElement>,
}

#[derive(Serialize, Deserialize)]
struct FreeOTPElement {
    algo: String,
    counter: u64,
    digits: u64,
    #[serde(rename = "issuerExt")]
    issuer_ext: String,
    #[serde(rename = "label")]
    _label: String,
    period: u64,
    secret: Vec<i8>,
    #[serde(rename = "type")]
    _type: String,
}

impl From<FreeOTPElement> for OTPElement {
    fn from(token: FreeOTPElement) -> Self {
        let counter: Option<u64> = if token.algo.to_uppercase().as_str() == "HOTP" {
            Some(token.counter)
        } else {
            None
        };
        OTPElement {
            counter,
            secret: encode_secret(&token.secret),
            issuer: token.issuer_ext,
            label: token._label,
            digits: token.digits,
            type_: OTPType::from(token._type.as_str()),
            algorithm: OTPAlgorithm::from(token.algo.as_str()),
            period: token.period,
            pin: None,
        }
    }
}

impl TryFrom<FreeOTPPlusJson> for Vec<OTPElement> {
    type Error = String;
    fn try_from(freeotp: FreeOTPPlusJson) -> Result<Self, Self::Error> {
        Ok(freeotp.tokens.into_iter().map(|e| e.into()).collect())
    }
}

fn encode_secret(secret: &[i8]) -> String {
    BASE32_NOPAD.encode(
        secret
            .iter()
            .map(|n| *n as u8)
            .collect::<Vec<u8>>()
            .as_slice(),
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        importers::importer::import_from_path,
        otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType},
    };

    use super::{encode_secret, FreeOTPPlusJson};

    #[test]
    fn test_secret_conversion() {
        let secret: Vec<i8> = vec![
            -34, 104, -37, -33, 82, -89, -93, -105, -14, 5, 100, -73, -84, 1, 11, 73, 101, -92,
            106, 122, -90, 111, -119, 30, 87, -6, 16, -57, -126, 25, 0, -65, -35, -76, -38,
        ];

        assert_eq!(
            encode_secret(&secret),
            String::from("3ZUNXX2SU6RZP4QFMS32YAILJFS2I2T2UZXYSHSX7IIMPAQZAC753NG2")
        );
    }

    #[test]
    fn test_conversion() {
        let imported = import_from_path::<FreeOTPPlusJson>(PathBuf::from(
            "test_samples/freeotp_plus_example1.json",
        ));

        assert!(imported.is_ok());
        assert_eq!(
            vec![
                OTPElement {
                    secret: "AAAAAAAAAAAAAAAA".to_string(),
                    issuer: "Example2".to_string(),
                    label: "Label2".to_string(),
                    digits: 6,
                    type_: OTPType::Totp,
                    algorithm: OTPAlgorithm::Sha1,
                    period: 30,
                    counter: None,
                    pin: None
                },
                OTPElement {
                    secret: "AAAAAAAA".to_string(),
                    issuer: "Example1".to_string(),
                    label: "Label1".to_string(),
                    digits: 6,
                    type_: OTPType::Totp,
                    algorithm: OTPAlgorithm::Sha256,
                    period: 30,
                    counter: None,
                    pin: None
                }
            ],
            imported.unwrap()
        )
    }
}
