use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::path::PathBuf;

use data_encoding::BASE32_NOPAD;

use utils::get_db_path;

use crate::crypto;
use crate::crypto::cryptography::gen_salt;
use crate::otp::otp_element::{OTPDatabase, OTPElement, OTPType};
use crate::utils::{self};
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

pub fn check_secret(secret: &str, type_: OTPType) -> Result<(), String> {
    match type_ {
        OTPType::Motp => match hex::decode(secret) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{}", e)),
        },
        _ => match BASE32_NOPAD.decode(secret.as_bytes()) {
            Ok(_r) => Ok(()),
            Err(error) => Err(format!("{}", error)),
        },
    }
}

pub fn export_database(path: PathBuf) -> Result<PathBuf, String> {
    let exported_path = if path.is_dir() {
        path.join("exported.cotp")
    } else {
        path
    };

    let encrypted_contents = match read_to_string(&get_db_path()) {
        Ok(result) => result,
        Err(e) => return Err(format!("Error during file reading: {:?}", e)),
    };
    let mut pw = utils::prompt_for_passwords("Password: ", 8, false);
    let contents = crypto::cryptography::decrypt_string(&encrypted_contents, &pw);
    pw.zeroize();
    match contents {
        Ok((mut contents, mut key, _salt)) => {
            key.zeroize();
            if contents == "[]" {
                return Err(String::from(
                    "there are no elements in your database, type \"cotp -h\" to get help",
                ));
            }
            let mut file = File::create(&exported_path).expect("Cannot create file");
            let contents_bytes = contents.as_bytes();
            file.write_all(contents_bytes)
                .expect("Failed to write contents");
            contents.zeroize();
            Ok(exported_path)
        }
        Err(e) => Err(e),
    }
}

pub fn overwrite_database(database: &OTPDatabase, password: &str) -> Result<(), std::io::Error> {
    let json_string: &str = &serde_json::to_string(&database)?;
    overwrite_database_json(json_string, password)
}

pub fn overwrite_database_json(json: &str, password: &str) -> Result<(), std::io::Error> {
    let salt = gen_salt().unwrap();
    let key = crypto::cryptography::argon_derive_key(password.as_bytes(), &salt).unwrap();
    overwrite_database_json_key(json, &key, salt.as_ref())
}

pub fn overwrite_database_key(
    database: &OTPDatabase,
    key: &Vec<u8>,
    salt: &[u8],
) -> Result<(), std::io::Error> {
    let json_string: &str = &serde_json::to_string(&database)?;
    overwrite_database_json_key(json_string, key, salt)
}

pub fn overwrite_database_json_key(
    json: &str,
    key: &Vec<u8>,
    salt: &[u8],
) -> Result<(), std::io::Error> {
    let encrypted =
        crypto::cryptography::encrypt_string_with_key(json.to_string(), key, salt).unwrap();
    let mut file = File::create(utils::get_db_path())?;
    match serde_json::to_string(&encrypted) {
        Ok(v) => utils::write_to_file(&v, &mut file),
        Err(e) => Err(std::io::Error::from(e)),
    }
}
