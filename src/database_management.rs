use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::path::PathBuf;

use data_encoding::BASE32_NOPAD;

use utils::{check_elements, get_db_path, millis_before_next_step};

use crate::crypto;
use crate::crypto::cryptography::gen_salt;
use crate::otp::otp_element::OTPElement;
use crate::otp::otp_helper::get_otp_code;
use crate::utils::{self, copy_string_to_clipboard, CopyType};
use zeroize::Zeroize;

type ReadResult = (Vec<OTPElement>, Vec<u8>, Vec<u8>);

pub fn get_elements() -> Result<ReadResult, String> {
    let mut pw = utils::prompt_for_passwords("Password: ", 8, false);
    let (elements, key, salt) = match read_from_file(&pw) {
        Ok((result, key, salt)) => (result, key, salt),
        Err(_e) => return Err(String::from("Cannot decrypt existing database")),
    };
    pw.zeroize();
    Ok((elements, key, salt))
}

pub fn print_elements_matching(issuer: Option<&str>, label: Option<&str>) -> Result<(), String> {
    let (elements, mut key, _salt) = get_elements()?;
    key.zeroize();

    elements
        .iter()
        .filter(|element| {
            (if let Some(i) = issuer {
                i.to_lowercase() == element.issuer().to_lowercase()
            } else {
                true
            }) && (if let Some(l) = label {
                l.to_lowercase() == element.label().to_lowercase()
            } else {
                true
            })
        })
        .for_each(|element| {
            let otp_code = match get_otp_code(element) {
                Ok(code) => code,
                Err(e) => e,
            };
            println!();
            println!("Issuer: {}", element.issuer());
            println!("Label: {}", element.label());
            println!(
                "OTP Code: {} ({} seconds remaining)",
                otp_code,
                millis_before_next_step() / 1000
            );
            match copy_string_to_clipboard(otp_code) {
                Ok(result) => match result {
                    CopyType::Native => println!("Copied to clipboard"),
                    CopyType::OSC52 => println!("Remote copied to clipboard"),
                },
                Err(()) => println!("Cannot copy to clipboard"),
            }
            println!();
        });
    Ok(())
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
            let mut vector: Vec<OTPElement> = match serde_json::from_str(&contents) {
                Ok(results) => results,
                Err(e) => {
                    contents.zeroize();
                    return Err(format!("Failed to deserialize database: {:?}", e));
                }
            };
            contents.zeroize();
            vector.sort_by_key(|a| a.issuer());
            Ok((vector, key, salt))
        }
        Err(e) => Err(e),
    }
}

pub fn check_secret(secret: &str, type_: &str) -> Result<(), String> {
    match type_ {
        "MOTP" => match hex::decode(secret) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{}", e)),
        },
        _ => match BASE32_NOPAD.decode(secret.as_bytes()) {
            Ok(_r) => Ok(()),
            Err(error) => Err(format!("{}", error)),
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub fn add_element(
    secret: &str,
    issuer: &str,
    label: &str,
    algorithm: &str,
    digits: u64,
    counter: Option<u64>,
    type_: &str,
    pin: Option<String>,
) -> Result<(), String> {
    let upper_secret = secret.to_uppercase().replace('=', "");
    match check_secret(&upper_secret, type_.to_uppercase().as_str()) {
        Ok(()) => {}
        Err(error) => return Err(error),
    }
    let mut pw = utils::prompt_for_passwords("Password: ", 8, false);
    let otp_element = OTPElement::new(
        upper_secret,
        issuer.to_string(),
        label.to_string(),
        digits,
        type_.to_string(),
        String::from(algorithm).to_uppercase(),
        30,
        counter,
        pin,
    );
    let mut elements = match read_from_file(&pw) {
        Ok((result, mut key, _salt)) => {
            key.zeroize();
            result
        }
        Err(e) => return Err(e),
    };
    elements.push(otp_element);
    let result = match overwrite_database(&elements, &pw) {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("{}", e)),
    };
    pw.zeroize();
    result
}

pub fn remove_element_from_db(indexes: Vec<usize>) -> Result<(), String> {
    if indexes.is_empty() {
        return Err(String::from("Bad args"));
    }

    let mut pw = utils::prompt_for_passwords("Password: ", 8, false);
    let mut elements: Vec<OTPElement> = match read_from_file(&pw) {
        Ok((result, mut key, _salt)) => {
            key.zeroize();
            result
        }
        Err(e) => {
            return Err(e);
        }
    };

    if indexes.iter().max().unwrap_or(&0) > &elements.len() {
        return Err(format!(
            "Index {} is out of bounds",
            indexes.iter().max().unwrap_or(&0)
        ));
    }
    let mut c = 0;

    for mut index in indexes {
        if index == 0 {
            return Err(String::from("0 is a bad index"));
        }
        //user inserts numbers starting from 1, so we will decrement the value because we use array indexes instead
        index -= 1;

        match check_elements(index - c, elements.as_slice()) {
            Ok(()) => {
                elements.remove(index - c);
                c += 1;
            }
            Err(e) => return Err(e),
        }
    }

    let result = match overwrite_database(&elements, &pw) {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("{}", e)),
    };
    pw.zeroize();
    result
}

pub fn edit_element(
    mut id: usize,
    secret: &str,
    issuer: &str,
    label: &str,
    algorithm: &str,
    digits: u64,
    counter: u64,
) -> Result<(), String> {
    if id == 0 {
        return Err(String::from("Invalid index"));
    }
    id -= 1;

    let mut pw = utils::prompt_for_passwords("Password: ", 8, false);
    let mut elements: Vec<OTPElement> = match read_from_file(&pw) {
        Ok((result, mut key, _salt)) => {
            key.zeroize();
            result
        }
        Err(_e) => return Err(String::from("Cannot decrypt existing database")),
    };

    let result = match check_elements(id, elements.as_slice()) {
        Ok(()) => {
            if !secret.trim().is_empty() {
                elements[id].set_secret(secret.to_string());
            }
            if !issuer.trim().is_empty() {
                elements[id].set_issuer(issuer.to_string());
            }
            if !label.trim().is_empty() {
                elements[id].set_label(label.to_string());
            }
            if !algorithm.trim().is_empty() {
                elements[id].set_algorithm(algorithm.to_string().to_uppercase());
            }
            if digits > 0 {
                elements[id].set_digits(digits);
            }
            if counter > 0 {
                elements[id].set_counter(Some(counter));
            }
            match overwrite_database(&elements, &pw) {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("{}", e)),
            }
        }
        Err(e) => Err(e),
    };
    pw.zeroize();
    result
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

pub fn show_qr_code(issuer: String) -> Result<(), String> {
    let (elements, mut key, _salt) = get_elements()?;
    key.zeroize();
    if let Some(element) = elements.iter().find(|value| {
        value
            .issuer()
            .to_lowercase()
            .contains(issuer.to_lowercase().as_str())
    }) {
        println!("{}", element.get_qrcode());
        Ok(())
    } else {
        Err(format!("Issuer \"{}\" not found", issuer))
    }
}

pub fn print_element_info(issuer: String) -> Result<(), String> {
    let (elements, mut key, _salt) = get_elements()?;
    key.zeroize();
    if elements.is_empty() {
        return Err(
            "there are no elements in your database. Type \"cotp -h\" to get help.".to_string(),
        );
    }

    if let Some(chosen_element) = elements.iter().find(|element| {
        element
            .issuer()
            .to_lowercase()
            .contains(issuer.to_lowercase().as_str())
    }) {
        println!("Issuer: {}", chosen_element.issuer());
        println!("Label: {}", chosen_element.label());
        println!("Algorithm: {}", chosen_element.algorithm());
        println!("Type: {}", chosen_element.type_());
        println!("Digits: {}", chosen_element.digits());
        Ok(())
    } else {
        Err(format!("Issuer \"{}\" not found", issuer))
    }
}

pub fn overwrite_database(elements: &[OTPElement], password: &str) -> Result<(), std::io::Error> {
    let json_string: &str = &serde_json::to_string(&elements)?;
    overwrite_database_json(json_string, password)
}

pub fn overwrite_database_json(json: &str, password: &str) -> Result<(), std::io::Error> {
    let salt = gen_salt().unwrap();
    let key = crypto::cryptography::argon_derive_key(password.as_bytes(), &salt).unwrap();
    overwrite_database_json_key(json, &key, salt.as_ref())
}

pub fn overwrite_database_key(
    elements: &[OTPElement],
    key: &Vec<u8>,
    salt: &[u8],
) -> Result<(), std::io::Error> {
    let json_string: &str = &serde_json::to_string(&elements)?;
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
