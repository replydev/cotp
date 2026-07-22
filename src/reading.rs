use crate::crypto;
use crate::otp::otp_element::{OTPDatabase, OTPElement};
use crate::path::DATABASE_PATH;
use crate::utils;
use color_eyre::eyre::{ErrReport, eyre};
use std::fs::read_to_string;
use std::io::{self, BufRead};
use zeroize::Zeroize;

pub type ReadResult = (OTPDatabase, Vec<u8>, Vec<u8>);

pub fn get_elements_from_input() -> color_eyre::Result<ReadResult> {
    let pw = utils::password("Password: ", 8);
    get_elements_with_password(pw)
}

pub fn get_elements_from_stdin() -> color_eyre::Result<ReadResult> {
    if let Some(password) = io::stdin().lock().lines().next() {
        return get_elements_with_password(password?);
    }
    Err(eyre!("Failure during stdin reading"))
}

fn get_elements_with_password(mut password: String) -> color_eyre::Result<ReadResult> {
    let (elements, key, salt) = read_from_file(&password)?;
    password.zeroize();
    Ok((elements, key, salt))
}

pub fn read_decrypted_text(password: &str) -> color_eyre::Result<(String, Vec<u8>, Vec<u8>)> {
    let encrypted_contents =
        read_to_string(DATABASE_PATH.get().unwrap()).map_err(ErrReport::from)?;
    if encrypted_contents.is_empty() {
        // Do not delete the file here: silently destroying a user file from a
        // read path is surprising and irreversible. An empty file can also be
        // the leftover of an interrupted write, in which case the user may
        // want to restore a backup instead of starting over.
        return Err(eyre!(
            "Your database file at {:?} is empty or corrupted. If you have a backup, restore it over that path; otherwise remove the file manually and restart cotp to initialize a new database.",
            DATABASE_PATH.get().unwrap()
        ));
    }
    //rust close files at the end of the function
    crypto::cryptography::decrypt_string(&encrypted_contents, password)
}

pub fn read_from_file(password: &str) -> color_eyre::Result<ReadResult> {
    match read_decrypted_text(password) {
        Ok((mut contents, key, salt)) => {
            let mut database: OTPDatabase = serde_json::from_str(&contents)
                .or_else(|_| serde_json::from_str::<Vec<OTPElement>>(&contents).map(Into::into))
                .map_err(ErrReport::from)?;
            contents.zeroize();
            database.sort();
            Ok((database, key, salt))
        }
        Err(e) => Err(e),
    }
}
