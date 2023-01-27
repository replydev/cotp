use std::fs::read_to_string;

use crate::otp::otp_element::OTPElement;

//no need to declare andOTP json struct cause it's the same as OTP element
pub fn import(filepath: &str) -> Result<Vec<OTPElement>, String> {
    let file_to_import_contents = match read_to_string(filepath) {
        Ok(result) => result,
        Err(e) => return Err(format!("Error during file reading: {e:?}")),
    };

    match serde_json::from_str::<Vec<OTPElement>>(&file_to_import_contents) {
        Ok(elements) => Ok(elements),
        Err(e) => Err(format!("Failed to serialize file: {e}")),
    }
}
