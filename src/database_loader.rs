use std::fs::{File, read_to_string};
use std::io::prelude::*;
use std::path::PathBuf;

use data_encoding::BASE32_NOPAD;
use serde_json;

use utils::{check_elements, get_db_path};

use crate::cryptography;
use crate::otp::otp_element::OTPElement;
use crate::utils;
use zeroize::Zeroize;

pub fn read_decrypted_text(password: &str) -> Result<String, String> {
    let encrypted_contents = match read_to_string(&get_db_path()) {
        Ok(result) => result,
        Err(e) => {
            // no need to zeroize since contents are encrypted
            return Err(format!("Error during file reading: {:?}",e));
        },
    };
    if encrypted_contents.len() == 0 {
        return match utils::delete_db() {
            Ok(_) => Err(String::from("Your database file was empty, please restart to create a new one.")),
            Err(_) => Err(String::from("Your database file is empty, please remove it manually and restart."))
        };
    }
    //rust close files at the end of the function
    cryptography::decrypt_string(&encrypted_contents, password)
}

pub fn read_from_file(password: &str) -> Result<Vec<OTPElement>, String> {
    return match read_decrypted_text(password) {
        Ok(mut contents) => {
            let vector: Vec<OTPElement> = match serde_json::from_str(&contents){
                Ok(results) => results,
                Err(e) => {
                    contents.zeroize();
                    return Err(format!("Failed to deserialize database: {:?}",e));
                },
            };
            contents.zeroize();
            Ok(vector)
        }
        Err(e) => {
            Err(e)
        }
    };
}

pub fn check_secret(secret: &str) -> Result<(), data_encoding::DecodeError> {
    return match BASE32_NOPAD.decode(secret.as_bytes()) {
        Ok(_r) => Ok(()),
        Err(error) => Err(error)
    };
}

pub fn add_element(secret: &String, issuer: &String, label: &String, algorithm: &str, digits: u64) -> Result<(), String> {
    let upper_secret = secret.to_uppercase().replace("=", "");
    match check_secret(&upper_secret) {
        Ok(()) => {}
        Err(error) => return Err(error.to_string())
    }
    let mut pw = cryptography::prompt_for_passwords("Password: ", 8, false);
    let otp_element = OTPElement::new(upper_secret.to_string(), issuer.to_string(), label.to_string(), digits, String::from("TOTP"), String::from(algorithm).to_uppercase(), String::from("Default"), 0, 0, 30, vec![]);
    let mut elements;
    match read_from_file(&pw) {
        Ok(result) => elements = result,
        Err(e) => return Err(e)
    }
    elements.push(otp_element);
    let result = match overwrite_database(elements, &pw) {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("{}", e))
    };
    pw.zeroize();
    result
}

pub fn remove_element_from_db(mut id: usize) -> Result<(), String> {
    if id == 0 {
        return Err(String::from("0 is a bad index"));
    }
    //user inserts numbers starting from 1, so we will decrement the value because we use array indexes instead
    id -= 1;

    let mut elements: Vec<OTPElement>;
    let mut pw = cryptography::prompt_for_passwords("Password: ", 8, false);
    match read_from_file(&pw) {
        Ok(result) => elements = result,
        Err(e) => {
            return Err(e);
        }
    }

    let result = match check_elements(id, &elements) {
        Ok(()) => {
            elements.remove(id);
            match overwrite_database(elements, &pw) {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("{}", e)),
            }
        }
        Err(e) => Err(e)
    };
    pw.zeroize();
    result
}

pub fn edit_element(mut id: usize, secret: &str, issuer: &str, label: &str, algorithm: &str, digits: u64) -> Result<(), String> {
    if id == 0 {
        return Err(String::from("Invalid index"));
    }
    id -= 1;

    let mut elements: Vec<OTPElement>;
    let mut pw = cryptography::prompt_for_passwords("Password: ", 8, false);
    match read_from_file(&pw) {
        Ok(result) => elements = result,
        Err(_e) => return Err(String::from("Cannot decrypt existing database"))
    }

    let result = match check_elements(id, &elements) {
        Ok(()) => {
            if secret != "" {
                elements[id].set_secret(secret.to_string());
            }
            if issuer != "." {
                elements[id].set_issuer(issuer.to_string());
            }
            if label != "." {
                elements[id].set_label(label.to_string());
            }
            if algorithm != "." {
                elements[id].set_algorithm(algorithm.to_string().to_uppercase());
            }
            if digits != 0 {
                elements[id].set_digits(digits);
            }
            match overwrite_database(elements, &pw) {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("{}", e)),
            }
        }
        Err(e) => Err(e)
    };
    pw.zeroize();
    result
}

pub fn export_database() -> Result<PathBuf, String> {
    let exported_path = utils::get_home_folder().join("exported.cotp");
    let mut file = File::create(&exported_path).expect("Cannot create file");
    let encrypted_contents = match read_to_string(&get_db_path()){
        Ok(result) => result,
        Err(e) => return Err(format!("Error during file reading: {:?}",e)),
    };
    let mut pw = cryptography::prompt_for_passwords("Password: ", 8, false);
    let contents = cryptography::decrypt_string(&encrypted_contents, &pw);
    pw.zeroize();
    return match contents {
        Ok(contents) => {
            if contents == "[]" {
                return Err(String::from("there are no elements in your database, type \"cotp -h\" to get help"));
            }
            file.write_all(contents.as_bytes()).expect("Failed to write contents");
            Ok(exported_path)
        }
        Err(e) => {
            Err(format!("{}", e))
        }
    };
}

pub fn overwrite_database(elements: Vec<OTPElement>, password: &str) -> Result<(), std::io::Error> {
    let json_string: &str = &serde_json::to_string(&elements)?;
    overwrite_database_json(json_string, password)
}

pub fn overwrite_database_json(json: &str, password: &str) -> Result<(), std::io::Error> {
    let encrypted = cryptography::encrypt_string(json.to_string(), password);
    let mut file = File::create(utils::get_db_path())?;
    utils::write_to_file(&encrypted, &mut file)
}

