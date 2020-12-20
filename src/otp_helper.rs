use otp::make_totp;
use super::database_loader;
use serde_json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct JsonResult{
    index: usize,
    issuer: String,
    label: String,
    otp_code: String,
}

impl JsonResult {
    pub fn new(index: usize, issuer: String, label: String,otp_code: String) -> JsonResult {
        JsonResult{
            index: index, 
            issuer: issuer,
            label: label,
            otp_code: otp_code
        }
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }
    pub fn set_issuer(&mut self,issuer: String){
        self.issuer = issuer;
    }
    pub fn set_label(&mut self,label: String){
        self.label = label;
    }
    pub fn set_otp_code(&mut self,otp_code: String){
        self.otp_code = otp_code;
    }
}

pub fn show_codes(){
    let elements: Vec<database_loader::OTPElement> = database_loader::read_from_file();
    for i in 0..elements.len() {
        print_totp(i,&elements[i]);
    }
}

fn print_totp(i: usize,element: &database_loader::OTPElement){
    if element.issuer() != ""{
        println!("{}) {} - {}: {}",i+1,element.issuer(),element.label(),get_good_otp_code(&element));
    }else{
        println!("{}) {}: {}",i+1,element.label(),get_good_otp_code(&element));
    }
}

fn get_good_otp_code(element: &database_loader::OTPElement) -> String {
    let otp = make_totp(
        &element.secret(), //we have replaced '=' in this method
               element.period(), 0).unwrap();
    let mut s_otp = otp.to_string();

    while s_otp.len() < element.digits() as usize {
        s_otp = String::from("0") + &s_otp;
    }
    s_otp
}

pub fn get_json_results() -> String{
    let elements: Vec<database_loader::OTPElement> = database_loader::read_from_file();

    let mut results: Vec<JsonResult> = Vec::new();

    for i in 0..elements.len() {
        let otp_code = get_good_otp_code(&elements[i]);
        results.push(JsonResult::new(i+1,elements[i].issuer(),elements[i].label(),otp_code))
    }

    let json_string: &str = &serde_json::to_string_pretty(&elements).unwrap();

    json_string.to_string()
}