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

    for element in vector {
        elements.push(OTPElement::new(
            element.secret,
            element.issuer.unwrap_or_default(),
            element.label.unwrap_or_default(),
            element.digits,
            element.type_,
            element.algorithm,
            String::from(""),
            0,
            0,
            30,
            element.counter,
            vec![]))
    }
    Ok(elements)
}
