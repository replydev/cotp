use std::fs::{File,read_to_string};
use std::io::prelude::*;
use serde_json;
use crate::utils;
use utils::{get_db_path,check_elements};
use crate::cryptograpy;
use crate::otp::otp_element::OTPElement;

pub fn read_decrypted_text(password: &str) -> Result<String,String>{
    let encrypted_contents = read_to_string(&get_db_path()).unwrap();
    //rust close files at the end of the function
    cryptograpy::decrypt_string(&encrypted_contents, password)
}

pub fn read_from_file(password: &str) -> Result<Vec<OTPElement>,String>{
    return match read_decrypted_text(password) {
        Ok(contents) => {
            let vector: Vec<OTPElement> = serde_json::from_str(&contents).unwrap();
            Ok(vector)
        },
        Err(e) => {
            Err(e)
        }
    }
}

pub fn check_secret(secret: &str) -> bool{
    //only uppercase characters and numbers
    if secret.is_empty(){
        return false;
    }
    // we have already uppercased the secret
    return secret.chars().all(char::is_alphanumeric);
}

pub fn add_element(secret: &String,issuer: &String,label: &String,algorithm: &str,digits: u64) -> Result<(),String>{
    let upper_secret = secret.to_uppercase().replace("=", "");
    if !check_secret(&upper_secret){
        return Err(String::from("Bad secret"))
    }
    let pw = &cryptograpy::prompt_for_passwords("Password: ",8,false);
    let otp_element = OTPElement::new(upper_secret.to_string(), issuer.to_string(), label.to_string(),digits, String::from("TOTP"), String::from(algorithm).to_uppercase(),String::from("Default"),0,0,30,vec![]);
    let mut elements;
    match read_from_file(pw){
        Ok(result) => elements = result,
        Err(e) => return Err(e)
    }
    elements.push(otp_element);
    match overwrite_database(elements,pw){
        Ok(()) => Ok(()),
        Err(e) => Err(format!("{}",e))
    }
}

pub fn remove_element_from_db(mut id: usize) -> Result<(),String>{
    if id == 0{
        return Err(String::from("0 is a bad index"));
    }
    //user inserts numbers starting from 1, so we will decrement the value becouse we use array indexes instead
    id -= 1;

    let mut elements: Vec<OTPElement>;
    let pw = &cryptograpy::prompt_for_passwords("Password: ",8,false);
    match read_from_file(pw){
        Ok(result) => elements = result,
        Err(e) => {
            return Err(e);
        }
    }

    match check_elements(id, &elements){
        Ok(()) => {
            for i in 0..elements.len(){
                if i == id {
                    elements.remove(i);
                    break;
                }
            }
            match overwrite_database(elements,pw){
                Ok(()) => Ok(()),
                Err(e) => Err(format!("{}",e)),
            }
        },
        Err(e) => Err(e)
    } 
}

pub fn edit_element(mut id: usize, secret: &str,issuer: &str,label: &str,algorithm: &str,digits: u64) -> Result<(), String> {
    if id == 0{
        return Err(String::from("Invalid element"));
    }
    id -= 1;

    let mut elements: Vec<OTPElement>;
    let pw = &cryptograpy::prompt_for_passwords("Password: ",8,false);
    match read_from_file(pw){
        Ok(result) => elements = result,
        Err(_e) => return Err(String::from("Cannot decrypt existing database"))
    }

    match check_elements(id,&elements){
        Ok(()) => {
            for i in 0..elements.len() {
                if i == id{
                    if secret != ""{
                        elements[i].set_secret(secret.to_string());
                    }
                    if issuer != "."{
                        elements[i].set_issuer(issuer.to_string());
                    }
                    if label != "."{
                        elements[i].set_label(label.to_string());
                    }
                    if algorithm != "."{
                        elements[i].set_algorithm(algorithm.to_string().to_uppercase());
                    }
                    if digits != 0{
                        elements[i].set_digits(digits);
                    }
                    break;
                }
            }
            match overwrite_database(elements,pw){
                Ok(()) => Ok(()),
                Err(e) => Err(format!("{}",e)),
            }
        },
        Err(e) => Err(e)
    }
}

pub fn export_database() -> Result<String, String> {
    let mut exported_path = utils::get_home_folder().to_str().unwrap().to_string();
    exported_path.push_str("/exported.cotp");
    let mut file = File::create(&exported_path).expect("Cannot create file");
    let encrypted_contents = read_to_string(&get_db_path()).unwrap();
    let contents = cryptograpy::decrypt_string(&encrypted_contents, &cryptograpy::prompt_for_passwords("Password: ",8,false));
    return match contents {
        Ok(contents) => {
            if contents == "[]" {
                return Err(String::from("there are no elements in your database, type \"cotp -h\" to get help"));
            }
            file.write_all(contents.as_bytes()).expect("Failed to write contents");
            Ok(exported_path)
        },
        Err(e) => {
            Err(format!("{}", e))
        }
    }
}

pub fn overwrite_database(elements: Vec<OTPElement>,password: &str) -> Result<(),std::io::Error>{
    let json_string: &str = &serde_json::to_string(&elements)?;
    overwrite_database_json(json_string,password)
}

pub fn overwrite_database_json(json: &str,password: &str) -> Result<(),std::io::Error>{
    let encrypted = cryptograpy::encrypt_string(json.to_string(), password);
    let mut file = File::create(utils::get_db_path())?;
    utils::write_to_file(&encrypted, &mut file)
}

