use std::fs::File;
use std::io::Read;
use serde_json;
use serde::{Deserialize, Serialize};
use super::utils;
use utils::get_db_path;


#[derive(Debug, Deserialize, Serialize)]
pub struct OTPElement {
    secret: String,
    label: String,
    algorithm: String
}

impl OTPElement {
    pub fn new(secret: String, label: String, algorithm: String) -> OTPElement {
        OTPElement {
            secret: secret,
            algorithm: algorithm,
            label: label
        }
    }
    pub fn secret(&self) -> String {
        self.secret.to_string().replace("=", "")
    }
    pub fn label(&self) -> String{
        self.label.to_string()
    }
    pub fn algorithm(&self) -> String{
        self.algorithm.to_string()
    }
}

fn read_from_file() -> Vec<OTPElement>{
    let mut file = File::open(&get_db_path()).expect("File not found!");
    //rust close files at the end of the function
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let vector: serde_json::Value = serde_json::from_str(&contents).unwrap();
    from_value_to_vec(vector)
}

fn from_value_to_vec(json_object: serde_json::Value) -> Vec<OTPElement>{
    let mut vector: Vec<OTPElement> = vec![];

    for i in 0..json_object.as_array().unwrap().len() {
        vector.push(
            OTPElement::new(
                json_object[i]["secret"].as_str().unwrap().to_string(),
                json_object[i]["label"].as_str().unwrap().to_string(),
                json_object[i]["algorithm"].as_str().unwrap().to_string(),
            )
        )
    }
    vector
}

pub fn get_elements() -> Vec<OTPElement> {
    let elements: Vec<OTPElement> = read_from_file();
    /*for i in 0..json_object.len() {
        elements.push(OTPElement{
            secret: json_object[i]["secret"].to_string(),
            label: json_object[i]["label"].to_string(),
            algorithm: json_object[i]["algorithm"].to_string()
        });
    }*/
    elements
}

pub fn remove_element_from_db(mut id: usize) -> bool{
    if id == 0{
        return false;
    }
    //user inserts numbers starting from 1, so we will decrement the value becouse we use array indexes instead
    id = id - 1;

    let mut elements: Vec<OTPElement> = read_from_file();

    if id >= elements.len(){
        return false;
    }

    for i in 0..elements.len(){
        if i == id {
            elements.remove(i);
        }
    }
    overwrite_database(elements);
    true
}


pub fn overwrite_database(elements: Vec<OTPElement>){
    let json_string: &str = &serde_json::to_string(&elements).unwrap();
    utils::write_to_file(json_string, &utils::get_db_path())
}

