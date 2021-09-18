use serde::{Deserialize, Serialize};
use serde_json;

use crate::{cryptography, database_loader};
use crate::otp::otp_element::OTPElement;
use crate::otp::otp_maker::make_totp;
use crate::utils::check_elements;

#[derive(Serialize, Deserialize)]
struct JsonResult {
    index: usize,
    issuer: String,
    label: String,
    otp_code: String,
}

impl JsonResult {
    pub fn new(index: usize, issuer: String, label: String, otp_code: String) -> JsonResult {
        JsonResult {
            index,
            issuer,
            label,
            otp_code,
        }
    }
}

pub fn read_codes() -> Result<Vec<OTPElement>, String> {
    match database_loader::read_from_file(&cryptography::prompt_for_passwords("Password: ", 8, false)) {
        Ok(result) => Ok(result),
        Err(e) => Err(e),
    }
}

pub fn list_codes(elements: &Vec<OTPElement>) {
    // used in single mode
    let mut i = 0;
    for element in elements {
        println!("{}) {} {} -> {}", i + 1, element.issuer(), element.label(), get_good_otp_code(element));
        i += 1;
    }
}

pub fn get_good_otp_code(element: &OTPElement) -> String {
    let otp = make_totp(
        &element.secret(), //we have replaced '=' in this method
        &element.algorithm().to_uppercase(), element.digits());

    "0".repeat(otp.len() - element.digits() as usize) + otp.as_str()
}

pub fn get_json_results() -> Result<String, String> {
    let elements: Vec<OTPElement>;

    match database_loader::read_from_file(&cryptography::prompt_for_passwords("Password: ", 8, false)) {
        Ok(result) => elements = result,
        Err(e) => return Err(e)
    }
    let mut results: Vec<JsonResult> = Vec::new();

    if elements.len() == 0 {
        return Err(String::from("there are no elements in your database, type \"cotp -h\" to get help"));
    }

    for i in 0..elements.len() {
        let otp_code = get_good_otp_code(&elements[i]);
        results.push(JsonResult::new(i + 1, elements[i].issuer(), elements[i].label(), otp_code))
    }

    let json_string: &str = &serde_json::to_string_pretty(&results).unwrap();

    Ok(json_string.to_string())
}

pub fn print_json_result(mut index: usize) -> Result<(), String> {
    if index == 0 {
        return Err(String::from("Invalid element"));
    }
    index -= 1;

    let elements: Vec<OTPElement>;

    match database_loader::read_from_file(&cryptography::prompt_for_passwords("Password: ", 8, false)) {
        Ok(result) => elements = result,
        Err(e) => return Err(e),
    }

    match check_elements(index, &elements) {
        Ok(()) => {}
        Err(e) => {
            return Err(e);
        }
    }

    let chosen_element: &OTPElement = &elements[index];

    println!("Issuer: {}", chosen_element.issuer());
    println!("Label: {}", chosen_element.label());
    println!("Algorithm: {}", chosen_element.algorithm());
    println!("Digits: {}", chosen_element.digits());
    Ok(())
}