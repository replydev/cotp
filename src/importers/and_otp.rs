use crate::database_loader::OTPElement;
use std::fs::read_to_string;

//no need to declarate andOTP json struct cause it's the same as OTP element

pub fn import(filepath: &str) -> Result<Vec<OTPElement>,String>{
    let file_to_import_contents = read_to_string(filepath).unwrap();
    let result: Result<Vec<OTPElement>,serde_json::Error> = serde_json::from_str(&file_to_import_contents);
    match result {
        Ok(element) => return Ok(element),
        Err(e) => return Err(String::from(format!("Failed to serialize file: {}",e)))
    }
}