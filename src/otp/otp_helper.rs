use prettytable::{Table, row, cell};
use serde_json;
use serde::{Deserialize, Serialize};
use crate::{cryptograpy, database_loader};
use crate::otp::otp_element::OTPElement;
use crate::otp::otp_maker::make_totp;
use crate::utils::check_elements;


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
}

pub fn read_codes() -> Result<Vec<OTPElement>,String>{
    match database_loader::read_from_file(&cryptograpy::prompt_for_passwords("Password: ", 8,false)){
        Ok(result) => Ok(result),
        Err(e) => Err(e),
    }
}

pub fn show_codes(elements: &Vec<OTPElement>){
    let mut table = Table::new();
    table.add_row(row!["Id","Issuer","Label","Code"]);
    for i in 0..elements.len() {
        add_element_to_table(i, &mut table, &elements[i]);
    }
    table.printstd();
}

fn add_element_to_table(i: usize, table: &mut Table,element: &OTPElement){
    table.add_row(row![i+1,element.issuer(),element.label(),get_good_otp_code(&element)]);
}

fn get_good_otp_code(element: &OTPElement) -> String {
    let otp = make_totp(
        &element.secret(), //we have replaced '=' in this method
               element.period(), 0,&element.algorithm().to_uppercase(),element.digits()).unwrap();
    let mut s_otp = otp.to_string();

    while s_otp.len() < element.digits() as usize {
        s_otp = String::from("0") + &s_otp;
    }
    s_otp
}

pub fn get_json_results() -> Result<String,String>{
    let elements: Vec<OTPElement>;

    match database_loader::read_from_file(&cryptograpy::prompt_for_passwords("Password: ",8,false)){
        Ok(result) => elements = result,
        Err(e) => return Err(e)
    }
    let mut results: Vec<JsonResult> = Vec::new();

    if elements.len() == 0{
        return Err(String::from("there are no elements in your database, type \"cotp -h\" to get help"));
    }

    for i in 0..elements.len() {
        let otp_code = get_good_otp_code(&elements[i]);
        results.push(JsonResult::new(i+1,elements[i].issuer(),elements[i].label(),otp_code))
    }

    let json_string: &str = &serde_json::to_string_pretty(&results).unwrap();

    Ok(json_string.to_string())
}

pub fn print_json_result(mut index: usize) -> Result<(),String>{
    if index == 0{
        return Err(String::from("Invalid element"));
    }
    index -= 1;

    let elements: Vec<OTPElement>;

    match database_loader::read_from_file(&cryptograpy::prompt_for_passwords("Password: ",8,false)){
        Ok(result) => elements = result,
        Err(e) => return Err(e),
    }

    match check_elements(index, &elements){
        Ok(()) => {},
        Err(e) => {
            return Err(e);
        }
    }

    let choosed_element: &OTPElement = &elements[index];

    println!("Issuer: {}",choosed_element.issuer());
    println!("Label: {}",choosed_element.label());
    println!("Algoritmh: {}",choosed_element.algorithm());
    println!("Digits: {}",choosed_element.digits());
    Ok(())
}