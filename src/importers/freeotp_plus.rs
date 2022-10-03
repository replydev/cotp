use std::fs::read_to_string;

use data_encoding::BASE32_NOPAD;
use serde::Deserialize;
use serde_json;

use crate::otp::otp_element::{OTPAlgorithm, OTPElement, OTPType};

#[derive(Deserialize)]
struct FreeOTPPlusJson {
    #[serde(rename = "tokenOrder")]
    token_order: Vec<String>,
    tokens: Vec<FreeOTPElement>,
}

#[derive(Deserialize)]
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

pub fn import(file_path: &str) -> Result<Vec<OTPElement>, String> {
    let json = match read_to_string(file_path) {
        Ok(r) => r,
        Err(e) => return Err(format!("{:?}", e)),
    };

    let freeotp: FreeOTPPlusJson = match serde_json::from_str(json.as_str()) {
        Ok(r) => r,
        Err(e) => return Err(format!("Error during deserializing: {:?}", e)),
    };

    Ok(freeotp
        .tokens
        .into_iter()
        .enumerate()
        .map(|(i, token)| {
            let counter: Option<u64> = if token.algo.to_uppercase().as_str() == "HOTP" {
                Some(token.counter)
            } else {
                None
            };
            OTPElement {
                counter,
                secret: encode_secret(&token.secret),
                issuer: token.issuer_ext,
                label: freeotp
                    .token_order
                    .get(i)
                    .unwrap_or(&String::from("No label"))
                    .to_owned(),
                digits: token.digits,
                type_: OTPType::from(token._type.as_str()),
                algorithm: OTPAlgorithm::from(token.algo.as_str()),
                period: token.period,
                pin: None,
            }
        })
        .collect())
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
    use super::encode_secret;

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
}
