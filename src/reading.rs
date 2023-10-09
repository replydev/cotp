use crate::crypto;
use crate::otp::otp_element::{OTPDatabase, OTPElement};
use crate::utils;
use color_eyre::eyre::{eyre, ErrReport};
use std::fs::read_to_string;
use utils::get_db_path;
use zeroize::Zeroize;

pub type ReadResult = (OTPDatabase, Vec<u8>, Vec<u8>);

pub fn get_elements() -> color_eyre::Result<ReadResult> {
    let mut pw = utils::password("Password: ", 8);
    let (elements, key, salt) = match read_from_file(&pw) {
        Ok((result, key, salt)) => (result, key, salt),
        Err(e) => return Err(ErrReport::from(e)),
    };
    pw.zeroize();
    Ok((elements, key, salt))
}

pub fn read_decrypted_text(password: &str) -> color_eyre::Result<(String, Vec<u8>, Vec<u8>)> {
    let encrypted_contents = read_to_string(get_db_path()).map_err(|e| ErrReport::from(e))?;
    if encrypted_contents.is_empty() {
        return match utils::delete_db() {
            Ok(_) => Err(eyre!(
                "Your database file was empty, please restart to create a new one.",
            )),
            Err(_) => Err(eyre!(
                "Your database file is empty, please remove it manually and restart.",
            )),
        };
    }
    //rust close files at the end of the function
    crypto::cryptography::decrypt_string(&encrypted_contents, password)
}

pub fn read_from_file(password: &str) -> color_eyre::Result<ReadResult> {
    match read_decrypted_text(password) {
        Ok((mut contents, key, salt)) => {
            let mut database: OTPDatabase = serde_json::from_str(&contents)
                .or_else(|_| serde_json::from_str::<Vec<OTPElement>>(&contents).map(|r| r.into()))
                .map_err(|e| ErrReport::from(e))?;
            contents.zeroize();
            database.sort();
            Ok((database, key, salt))
        }
        Err(e) => Err(e),
    }
}
