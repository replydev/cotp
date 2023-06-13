use crate::crypto;
use crate::otp::otp_element::{OTPDatabase, OTPElement};
use crate::utils;
use std::fs::read_to_string;
use utils::get_db_path;
use zeroize::Zeroize;

pub type ReadResult = (OTPDatabase, Vec<u8>, Vec<u8>);

pub fn get_elements() -> Result<ReadResult, String> {
    let mut pw = utils::password("Password: ", 8);
    let (elements, key, salt) = match read_from_file(&pw) {
        Ok((result, key, salt)) => (result, key, salt),
        Err(e) => return Err(format!("Cannot decrypt existing database: {e}")),
    };
    pw.zeroize();
    Ok((elements, key, salt))
}

pub fn read_decrypted_text(password: &str) -> Result<(String, Vec<u8>, Vec<u8>), String> {
    let encrypted_contents =
        read_to_string(get_db_path()).map_err(|e| format!("Error during file reading: {e:?}"))?;
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
            let mut database: OTPDatabase = serde_json::from_str(&contents)
                .or_else(|_| serde_json::from_str::<Vec<OTPElement>>(&contents).map(|r| r.into()))
                .map_err(|e| format!("Failed to deserialize database: {e:?}"))?;
            contents.zeroize();
            database.sort();
            Ok((database, key, salt))
        }
        Err(e) => Err(e),
    }
}
