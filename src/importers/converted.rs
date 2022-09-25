use std::fs::read_to_string;

use serde::Deserialize;
use serde_json;

use crate::otp::otp_element::OTPElement;

#[derive(Deserialize)]
struct ConvertedJson {
    label: Option<String>,
    secret: String,
    issuer: Option<String>,
    #[serde(rename = "type")]
    type_: String,
    algorithm: String,
    digits: u64,
    counter: u64,
}

pub fn import(filepath: &str) -> Result<Vec<OTPElement>, String> {
    let file_to_import_contents = match read_to_string(filepath) {
        Ok(result) => result,
        Err(e) => return Err(format!("Error during file reading: {:?}", e)),
    };
    let result: Result<Vec<ConvertedJson>, serde_json::Error> =
        serde_json::from_str(&file_to_import_contents);
    let vector: Vec<ConvertedJson> = match result {
        Ok(r) => r,
        Err(e) => return Err(format!("{}", e)),
    };

    Ok(vector
        .into_iter()
        .map(|e| {
            let counter: Option<u64> = if e.type_.to_uppercase().as_str() == "HOTP" {
                Some(e.counter)
            } else {
                None
            };
            OTPElement::new(
                e.secret,
                e.issuer.unwrap_or_default(),
                e.label.unwrap_or_default(),
                e.digits,
                e.type_,
                e.algorithm,
                0,
                counter,
                None,
            )
        })
        .collect())
}
