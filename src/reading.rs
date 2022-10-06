use crate::crypto;
use crate::otp::otp_element::{OTPDatabase, OTPElement};
use crate::utils::{self};
use std::fs::read_to_string;
use utils::get_db_path;
use zeroize::Zeroize;

pub type ReadResult = (OTPDatabase, Vec<u8>, Vec<u8>);

pub fn get_elements() -> Result<ReadResult, String> {
    let mut pw = utils::prompt_for_passwords("Password: ", 8, false);
    let (elements, key, salt) = match read_from_file(&pw) {
        Ok((result, key, salt)) => (result, key, salt),
        Err(e) => return Err(format!("Cannot decrypt existing database: {}", e)),
    };
    pw.zeroize();
    Ok((elements, key, salt))
}

pub fn read_decrypted_text(password: &str) -> Result<(String, Vec<u8>, Vec<u8>), String> {
    let encrypted_contents = match read_to_string(&get_db_path()) {
        Ok(result) => result,
        Err(e) => {
            // no need to zeroize since contents are encrypted
            return Err(format!("Error during file reading: {:?}", e));
        }
    };
    if encrypted_contents.is_empty() {
        return match utils::delete_db() {
            Ok(_) => Err(String::from(
                "Your database file was empty, please restart to create a new one.",
            )),
            Err(_) => Err(String::from(
                "Your database file is empty, please remove it manually and restart.",
            )),
        };
    }
    //rust close files at the end of the function
    crypto::cryptography::decrypt_string(&encrypted_contents, password)
}

pub fn read_from_file(password: &str) -> Result<ReadResult, String> {
    match read_decrypted_text(password) {
        Ok((mut contents, key, salt)) => {
            let mut database: OTPDatabase = match serde_json::from_str(&contents) {
                Ok(results) => results,
                Err(e) => {
                    let elements: Vec<OTPElement> = match serde_json::from_str(&contents) {
                        Ok(r) => r,
                        Err(_) => {
                            contents.zeroize();
                            return Err(format!("Failed to deserialize database: {:?}", e));
                        }
                    };
                    OTPDatabase::new(1, elements)
                }
            };
            contents.zeroize();
            database.sort();
            Ok((database, key, salt))
        }
        Err(e) => Err(e),
    }
}
