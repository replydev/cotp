use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Serialize};

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

#[derive(Serialize, Deserialize)]
pub struct FreeOTPPlusJson {
    #[serde(rename = "tokenOrder")]
    token_order: Vec<String>,
    tokens: Vec<FreeOTPElement>,
}

impl FreeOTPPlusJson {
    /// Creates a new instance of FreeOTPPlusJSON. Currently we clone the tokens label to retrieve the tokens order.
    pub fn new(tokens: Vec<FreeOTPElement>) -> Self {
        let token_order: Vec<String> = tokens
            .iter()
            .map(|e| {
                if e.issuer_ext.is_empty() {
                    e._label.clone()
                } else {
                    format!("{}:{}", e.issuer_ext, e._label)
                }
            })
            .collect();

        Self {
            token_order,
            tokens,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FreeOTPElement {
    pub algo: String,
    pub counter: u64,
    pub digits: u64,
    #[serde(rename = "issuerExt")]
    pub issuer_ext: String,
    #[serde(rename = "label")]
    pub _label: String,
    pub period: u64,
    pub secret: Vec<i8>,
    #[serde(rename = "type")]
    pub _type: String,
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
        importers::{freeotp_plus::FreeOTPElement, importer::import_from_path},
        otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType},
    };

    use std::fs;

    use crate::otp::otp_element::OTPDatabase;
    use color_eyre::Result;

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

    #[test]
    fn test_freeotp_export() {
        // Arrange
        let input_json: String = fs::read_to_string(PathBuf::from("test_samples/cotp_input.json"))
            .expect("Cannot read input file for test");
        let input_cotp_database: OTPDatabase =
            serde_json::from_str(input_json.as_str()).expect("Cannot deserialize into input JSON");

        // Act
        let converted: Result<FreeOTPPlusJson> = (&input_cotp_database).try_into();

        // Assert
        assert!(converted.is_ok());

        let free_otp = converted.unwrap();

        assert_eq!(
            vec!["label1".to_string(), "ciccio:label2".to_string()],
            free_otp.token_order
        );

        assert_eq!(
            vec![
                FreeOTPElement {
                    algo: "SHA1".to_string(),
                    counter: 0,
                    digits: 6,
                    issuer_ext: String::default(),
                    _label: "label1".to_string(),
                    period: 30,
                    secret: vec![0],
                    _type: "TOTP".to_string()
                },
                FreeOTPElement {
                    algo: "SHA256".to_string(),
                    counter: 3,
                    digits: 6,
                    issuer_ext: "ciccio".to_string(),
                    _label: "label2".to_string(),
                    period: 30,
                    secret: vec![0],
                    _type: "HOTP".to_string()
                }
            ],
            free_otp.tokens
        )
    }
}
