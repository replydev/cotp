use std::fs::File;
use std::io::Read;
use json;

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

pub fn read_from_file(filename: String) -> Vec<OTPElement> {
    let mut elements: Vec<OTPElement> = Vec::new();
    let mut file = File::open(filename).expect("File not found!");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let json_object = json::parse(&contents).unwrap();
    for i in 0..json_object.len() {
        elements.push(OTPElement{
            secret: json_object[i]["secret"].to_string(),
            label: json_object[i]["label"].to_string(),
            algorithm: json_object[i]["algorithm"].to_string()
        });
    }
    //rust close files at the end of the function
    elements
}

