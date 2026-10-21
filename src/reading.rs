use crate::crypto;
use crate::otp::otp_element::{OTPDatabase, OTPElement};
use crate::path::DATABASE_PATH;
use color_eyre::eyre::{ErrReport, eyre};
use std::fs::read_to_string;
use zeroize::Zeroizing;

pub type ReadResult = (OTPDatabase, Vec<u8>, Vec<u8>);

pub fn read_decrypted_text(password: &str) -> color_eyre::Result<(String, Vec<u8>, Vec<u8>)> {
    let encrypted_contents =
        read_to_string(DATABASE_PATH.get().unwrap()).map_err(ErrReport::from)?;
    if encrypted_contents.is_empty() {
        return Err(eyre!(
            "Vault file is empty. Restore a verified backup; the file was not changed."
        ));
    }
    //rust close files at the end of the function
    crypto::cryptography::decrypt_string(&encrypted_contents, password)
}

pub fn read_from_file(password: &str) -> color_eyre::Result<ReadResult> {
    match read_decrypted_text(password) {
        Ok((contents, key, salt)) => {
            let contents = Zeroizing::new(contents);
            let key = Zeroizing::new(key);
            let mut database: OTPDatabase = serde_json::from_str(&contents)
                .or_else(|_| serde_json::from_str::<Vec<OTPElement>>(&contents).map(Into::into))
                .map_err(ErrReport::from)?;
            drop(contents);
            crate::otp::migrations::migrate(&mut database)?;
            database.sort();
            Ok((database, key.to_vec(), salt))
        }
        Err(e) => Err(e),
    }
}
