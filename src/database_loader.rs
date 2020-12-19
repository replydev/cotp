use std::fs::{File,read_to_string};
use std::io::prelude::*;
use serde_json;
use serde::{Deserialize, Serialize};
use super::utils;
use utils::get_db_path;
extern crate regex;
use regex::Regex;
use super::cryptograpy;

#[derive(Serialize, Deserialize)]
pub struct OTPElement {
    secret: String,
    issuer: String,
    label: String,
    digits: u64,
    #[serde(rename = "type")]
    _type: String,
    algorithm: String,
    thumbnail: String,
    last_used: u64,
    used_frequency: u64,
    period: u64,
    tags: Vec<String>,
}

impl OTPElement {
    pub fn new(secret: String,issuer: String,label: String,digits: u64,_type: String,algorithm: String,thumbnail: String,last_used: u64,used_frequency: u64,period: u64,tags: Vec<String>,
    ) -> OTPElement {
        OTPElement {
            secret,
            issuer,
            label,
            digits,
            _type,
            algorithm,
            thumbnail,
            last_used,
            used_frequency,
            period,
            tags,
        }
    }
    pub fn secret(&self) -> String {
        self.secret.to_string().replace("=", "")
    }
    pub fn label(&self) -> String{
        self.label.to_string()
    }
    pub fn issuer(&self) -> String{
        self.issuer.to_string()
    }
    pub fn period(&self) -> u64{
        self.period
    }

    pub fn set_secret(&mut self,secret: String){
        self.secret = secret;
    }
    pub fn set_label(&mut self,label: String){
        self.label = label;
    }
    pub fn set_issuer(&mut self,issuer: String){
        self.issuer = issuer;
    }
}

pub fn read_from_file() -> Vec<OTPElement>{
    let mut encrypted_contents = read_to_string(&get_db_path()).unwrap();
    //rust close files at the end of the function
    let contents = cryptograpy::decrypt_string(&mut encrypted_contents, &cryptograpy::prompt_for_passwords("Password: "));
    match contents {
        Ok(contents) => {
            let vector: Vec<OTPElement> = serde_json::from_str(&contents).unwrap();
            return vector;
        },
        Err(e) => {
            println!("{}", e);
            return Vec::new();
        }
    }
}

pub fn check_secret(secret: &str) -> bool{
    let regex = Regex::new(r"^[A-Z0-9.]+$").unwrap(); //only uppercase characters and numbers
    regex.is_match(secret)
}

pub fn add_element(secret: &String,issuer: &String,label: &String) -> bool{
    if !check_secret(&secret){
        return false;
    }
    let otp_element = OTPElement::new(secret.to_string(), issuer.to_string(), label.to_string(),6, String::from("TOTP"), String::from("SHA1"),String::from("Default"),0,0,30,vec![]);
    let mut elements = read_from_file();
    elements.push(otp_element);
    overwrite_database(elements);
    true
}

pub fn remove_element_from_db(mut id: usize) -> bool{
    if id == 0{
        return false;
    }
    //user inserts numbers starting from 1, so we will decrement the value becouse we use array indexes instead
    id -= 1;

    let mut elements: Vec<OTPElement> = read_from_file();

    if id >= elements.len(){
        return false;
    }

    for i in 0..elements.len(){
        if i == id {
            elements.remove(i);
            break;
        }
    }
    overwrite_database(elements);
    true
}

pub fn modify_element(mut id: usize, secret: &str,issuer: &str,label: &str) -> Result<(), String> {
    if id == 0{
        return Err(String::from("Invalid element"));
    }
    id -= 1;
    let mut elements: Vec<OTPElement> = read_from_file();

    if id >= elements.len() {
        return Err(String::from("Invalid element"));
    }

    for i in 0..elements.len() {
        if i == id{
            if secret != "."{
                elements[i].set_secret(secret.to_string());
            }
            if issuer != "."{
                elements[i].set_issuer(issuer.to_string());
            }
            if label != "."{
                elements[i].set_label(label.to_string());
            }
            break;
        }
    }
    overwrite_database(elements);
    Ok(())
}

pub fn export_database() -> Result<String, String> {
    let exported_path = utils::get_home_folder() + "/exported.cotp";
    let mut file = File::create(&exported_path).expect("Cannot create file");
    let mut encrypted_contents = read_to_string(&get_db_path()).unwrap();
    let contents = cryptograpy::decrypt_string(&mut encrypted_contents, &cryptograpy::prompt_for_passwords("Password: "));
    match contents {
        Ok(contents) => {
            file.write_all(contents.as_bytes()).expect("Failed to write contents");
            return Ok(exported_path);
        },
        Err(e) => {
            return Err(format!("{}",e));
        }
    }
}

pub fn overwrite_database(elements: Vec<OTPElement>){
    let json_string: &str = &serde_json::to_string(&elements).unwrap();
    let encrypted = cryptograpy::encrypt_string(&mut json_string.to_string(), &cryptograpy::prompt_for_passwords("Insert password for database encryption: "));
    utils::write_to_file(&encrypted, &mut File::create(utils::get_db_path()).expect("Failed to open file"));
}

