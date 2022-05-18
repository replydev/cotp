use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::path::PathBuf;

use data_encoding::BASE32_NOPAD;

use utils::{check_elements, get_db_path, millis_before_next_step};

use crate::cryptography;
use crate::otp::otp_element::OTPElement;
use crate::otp::otp_helper::get_otp_code;
use crate::utils;
use copypasta_ext::prelude::ClipboardProvider;
use copypasta_ext::x11_fork::ClipboardContext;
use zeroize::Zeroize;

pub fn get_elements() -> Result<Vec<OTPElement>, String> {
    let mut pw = utils::prompt_for_passwords("Password: ", 8, false);
    let elements: Vec<OTPElement> = match read_from_file(&pw) {
        Ok(result) => result,
        Err(_e) => return Err(String::from("Cannot decrypt existing database")),
    };
    pw.zeroize();
    Ok(elements)
}

pub fn print_elements_matching(issuer: Option<&str>, label: Option<&str>) -> Result<(), String> {
    let elements = get_elements()?;

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
            if let Ok(mut ctx) = ClipboardContext::new() {
                match ctx.set_contents(otp_code) {
                    Ok(_) => println!("Copied to clipboard"),
                    Err(_) => println!("Cannot copy OTP Code to clipboard"),
                };
            }
            println!();
        });
    Ok(())
}

pub fn read_decrypted_text(password: &str) -> Result<String, String> {
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
    cryptography::decrypt_string(&encrypted_contents, password)
}

pub fn read_from_file(password: &str) -> Result<Vec<OTPElement>, String> {
    return match read_decrypted_text(password) {
        Ok(mut contents) => {
            let mut vector: Vec<OTPElement> = match serde_json::from_str(&contents) {
                Ok(results) => results,
                Err(e) => {
                    contents.zeroize();
                    return Err(format!("Failed to deserialize database: {:?}", e));
                }
            };
            contents.zeroize();
            vector.sort_by_key(|a| a.issuer());
            Ok(vector)
        }
        Err(e) => Err(e),
    };
}

pub fn check_secret(secret: &str) -> Result<(), data_encoding::DecodeError> {
    return match BASE32_NOPAD.decode(secret.as_bytes()) {
        Ok(_r) => Ok(()),
        Err(error) => Err(error),
    };
}

pub fn add_element(
    secret: &str,
    issuer: &str,
    label: &str,
    algorithm: &str,
    digits: u64,
    counter: u64,
    hotp_type: bool,
) -> Result<(), String> {
    let upper_secret = secret.to_uppercase().replace('=', "");
    match check_secret(&upper_secret) {
        Ok(()) => {}
        Err(error) => return Err(error.to_string()),
    }
    let mut pw = utils::prompt_for_passwords("Password: ", 8, false);
    let type_ = if hotp_type { "HOTP" } else { "TOTP" };
    let otp_element = OTPElement::new(
        upper_secret,
        issuer.to_string(),
        label.to_string(),
        digits,
        type_.to_string(),
        String::from(algorithm).to_uppercase(),
        String::from("Default"),
        0,
        0,
        30,
        counter,
        vec![],
    );
    let mut elements = match read_from_file(&pw) {
        Ok(result) => result,
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
        Ok(result) => result,
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
        Ok(result) => result,
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
    let contents = cryptography::decrypt_string(&encrypted_contents, &pw);
    pw.zeroize();
    return match contents {
        Ok(mut contents) => {
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
    };
}

pub fn show_qr_code(issuer: String) -> Result<(), String> {
    let elements = get_elements()?;
    if let Some(element) = elements
        .into_iter()
        .filter(|value| {
            value
                .issuer()
                .to_lowercase()
                .contains(issuer.to_lowercase().as_str())
        })
        .next()
    {
        println!("{}", element.get_qrcode());
        Ok(())
    } else {
        Err(format!("Issuer \"{}\" not found", issuer))
    }
}

pub fn print_element_info(issuer: String) -> Result<(), String> {
    let elements = get_elements()?;
    if elements.is_empty() {
        return Err(
            "there are no elements in your database. Type \"cotp -h\" to get help.".to_string(),
        );
    }

    if let Some(chosen_element) = elements
        .iter()
        .filter(|element| {
            element
                .issuer()
                .to_lowercase()
                .contains(issuer.to_lowercase().as_str())
        })
        .next()
    {
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
    let encrypted = cryptography::encrypt_string(json.to_string(), password).unwrap();
    let mut file = File::create(utils::get_db_path())?;
    utils::write_to_file(&encrypted, &mut file)
}
