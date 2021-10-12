use std::fs::read_to_string;

use crate::otp::otp_element::OTPElement;

//no need to declare andOTP json struct cause it's the same as OTP element

pub fn import(filepath: &str) -> Result<Vec<OTPElement>, String> {
    let file_to_import_contents = match read_to_string(filepath) {
        Ok(result) => result,
        Err(e) => return Err(format!("Error during file reading: {:?}",e)),
    };
    let result: Result<Vec<OTPElement>, serde_json::Error> = serde_json::from_str(&file_to_import_contents);
    return match result {
        Ok(element) => Ok(element),
        Err(e) => Err(String::from(format!("Failed to serialize file: {}", e)))
    };
}