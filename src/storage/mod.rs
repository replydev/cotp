//! Persistence layer for the OTP database.
//!
//! [`OTPDatabase`] holds only domain data; this module owns everything about
//! moving it to and from disk: password prompting, decryption and
//! deserialization on load (including the legacy v1 format fallback), and
//! migration, encryption and the actual filesystem write on save.

use std::fs::{File, read_to_string};
use std::io::{self, BufRead, Write};
use std::path::Path;

use eyre::{ErrReport, eyre};
use zeroize::Zeroize;

use crate::crypto::cryptography::{
    argon_derive_key, decrypt_string, encrypt_string_with_key, gen_salt,
};
use crate::otp::migrations::migrate;
use crate::otp::otp_element::{OTPDatabase, OTPElement};
use crate::path::DATABASE_PATH;
use crate::utils;

pub type ReadResult = (OTPDatabase, Vec<u8>, Vec<u8>);

pub fn get_elements_from_input() -> eyre::Result<ReadResult> {
    let pw = utils::try_password("Password: ", 8)?;
    get_elements_with_password(pw)
}

pub fn get_elements_from_stdin() -> eyre::Result<ReadResult> {
    if let Some(password) = io::stdin().lock().lines().next() {
        return get_elements_with_password(password?);
    }
    Err(eyre!("Failure during stdin reading"))
}

fn get_elements_with_password(mut password: String) -> eyre::Result<ReadResult> {
    let read_result = read_from_file(DATABASE_PATH.get().unwrap(), &password);
    password.zeroize();
    read_result
}

fn read_decrypted_text(path: &Path, password: &str) -> eyre::Result<(String, Vec<u8>, Vec<u8>)> {
    let encrypted_contents = read_to_string(path).map_err(ErrReport::from)?;
    if encrypted_contents.is_empty() {
        // Do not delete the file here: silently destroying a user file from a
        // read path is surprising and irreversible. An empty file can also be
        // the leftover of an interrupted write, in which case the user may
        // want to restore a backup instead of starting over.
        return Err(eyre!(
            "Your database file at {path:?} is empty or corrupted. If you have a backup, restore it over that path; otherwise remove the file manually and restart cotp to initialize a new database.",
        ));
    }
    //rust close files at the end of the function
    decrypt_string(&encrypted_contents, password)
}

fn read_from_file(path: &Path, password: &str) -> eyre::Result<ReadResult> {
    let (mut contents, key, salt) = read_decrypted_text(path, password)?;
    let mut database: OTPDatabase = serde_json::from_str(&contents)
        .or_else(|_| serde_json::from_str::<Vec<OTPElement>>(&contents).map(Into::into))
        .map_err(ErrReport::from)?;
    contents.zeroize();
    database.sort();
    Ok((database, key, salt))
}

/// Encrypts the database with the given key and writes it to `path`.
///
/// `migrate()` runs on every save, so the written database is always at the
/// current schema version. The modified flag is cleared only AFTER a
/// successful write, so a failed save leaves the database marked dirty.
///
/// Clearing the flag on success is also what makes the `passwd` flow safe:
/// `passwd` persists the database itself via [`save_with_pw`] (new salt + key
/// derived from the new password), and the cleared flag makes the final
/// `is_modified()` check in `main()` skip its own save — which would
/// otherwise re-encrypt the database with the OLD key and silently undo the
/// password change.
pub fn save(
    database: &mut OTPDatabase,
    key: &Vec<u8>,
    salt: &[u8],
    path: &Path,
) -> eyre::Result<()> {
    migrate(database)?;
    encrypt_and_write(database, key, salt, path)?;
    database.clear_modified();
    Ok(())
}

/// Derives a fresh salt + key from `password` and saves the database with
/// them, returning both so the caller can keep using the new key.
pub fn save_with_pw(
    database: &mut OTPDatabase,
    password: &str,
    path: &Path,
) -> eyre::Result<(Vec<u8>, [u8; 16])> {
    let salt = gen_salt()?;
    let key = argon_derive_key(password.as_bytes(), &salt)?;
    save(database, &key, &salt, path)?;
    Ok((key, salt))
}

fn encrypt_and_write(
    database: &OTPDatabase,
    key: &Vec<u8>,
    salt: &[u8],
    path: &Path,
) -> eyre::Result<()> {
    // The plaintext JSON contains every secret in the database: wipe it
    // from memory as soon as it has been encrypted
    let mut json = serde_json::to_string(database)?;
    let encrypted = encrypt_string_with_key(&json, key, salt);
    json.zeroize();
    let encrypted = encrypted?;
    let mut file = create_database_file(path)?;
    let content = serde_json::to_string(&encrypted)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

/// Creates (or truncates) the database file.
///
/// On unix the file is created with mode 0600 so other users cannot read it.
/// The database is encrypted, so this is defense in depth rather than a
/// confidentiality requirement.
#[cfg(unix)]
fn create_database_file(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn create_database_file(path: &Path) -> std::io::Result<File> {
    File::create(path)
}
