use std::fs::read_to_string;

use serde::Deserialize;
use serde_json;

use crate::otp::otp_element::OTPElement;

#[derive(Deserialize)]
struct ConvertedJson {
    label: Option<String>,
    secret: String,
    issuer: Option<String>,
    #[serde(rename="type")]
    type_: String,
    algorithm: String,
    digits: u64,
    counter: u64
}

pub fn import(filepath: &str) -> Result<Vec<OTPElement>, String> {
    let file_to_import_contents = match read_to_string(filepath) {
        Ok(result) => result,
        Err(e) => return Err(format!("Error during file reading: {:?}",e)),
    };
    let result: Result<Vec<ConvertedJson>, serde_json::Error> = serde_json::from_str(&file_to_import_contents);
    let vector: Vec<ConvertedJson>;

    match result {
        Ok(r) => vector = r,
        Err(e) => return Err(format!("{}", e)),
    }

    let mut elements: Vec<OTPElement> = Vec::new();

    for i in 0..vector.len() {
        let secret = vector[i].secret.to_owned();
        let issuer = vector[i].issuer.to_owned().unwrap_or_default();
        let label = vector[i].label.to_owned().unwrap_or_default();
        let digits = vector[i].digits;
        let counter = vector[i].counter;
        let algorithm = vector[i].algorithm.to_owned();
        let type_ = vector[i].type_.to_owned();
        elements.push(OTPElement::new(
            secret,
            issuer,
            label,
            digits,
            type_,
            algorithm,
            String::from(""),
            0,
            0,
            30,
            counter,
            vec![]))
    }
    Ok(elements)
}
