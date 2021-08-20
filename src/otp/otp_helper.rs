use prettytable::{Table, row, cell,format};
use serde_json;
use serde::{Deserialize, Serialize};
use crate::{cryptography, database_loader};
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
            index,
            issuer,
            label,
            otp_code
        }
    }
}

pub struct PrintSettings {
    max_id: usize,
    max_issuer: usize,
    max_label: usize,
    max_code: usize,
}

impl PrintSettings {
    pub fn new() -> PrintSettings{
        // set the length of id, issuer, label, and code words
        PrintSettings {
            max_id: 2,
            max_issuer: 6,
            max_label: 5,
            max_code: 4,
        }
    }

    pub fn check_other(&mut self,other: &PrintSettings){
        if other.max_id > self.max_id {
            self.max_id = other.max_id;
        }
        if other.max_issuer > self.max_issuer {
            self.max_issuer = other.max_issuer;
        }
        if other.max_label > self.max_label {
            self.max_label = other.max_label;
        }
        if other.max_code > self.max_code {
            self.max_code = other.max_code;
        }
    }

    pub fn get_width(&self) -> usize {
        self.max_id + 2 + self.max_issuer + 2 + self.max_label + 2 + self.max_code + 2 + 2
    }
}

pub fn read_codes() -> Result<Vec<OTPElement>,String>{
    match database_loader::read_from_file(&cryptography::prompt_for_passwords("Password: ", 8,false)){
        Ok(result) => Ok(result),
        Err(e) => Err(e),
    }
}

pub fn show_codes(elements: &Vec<OTPElement>) -> usize{
    let mut print_settings = PrintSettings::new();
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row!["Id","Issuer","Label","Code"]);
    for i in 0..elements.len() {
        add_element_to_table(i, &mut table, &elements[i],&mut print_settings);
    }
    table.printstd();
    print_settings.get_width()
}

fn add_element_to_table(i: usize, table: &mut Table,element: &OTPElement,print_settings: &mut PrintSettings){
    let index = (i+1).to_string();
    let issuer = element.issuer();
    let label = element.label();
    let code = get_good_otp_code(&element);
    table.add_row(row![index,issuer,label,code]);

    let mut temp_print = PrintSettings::new();
    temp_print.max_id = index.chars().count();
    temp_print.max_issuer = issuer.chars().count();
    temp_print.max_label = label.chars().count();
    temp_print.max_code = code.chars().count();

    print_settings.check_other(&temp_print);
}

fn get_good_otp_code(element: &OTPElement) -> String {
    let otp = make_totp(
        &element.secret(), //we have replaced '=' in this method
               &element.algorithm().to_uppercase(),element.digits());

    "0".repeat(otp.len() - element.digits() as usize) + otp.as_str()
}

pub fn get_json_results() -> Result<String,String>{
    let elements: Vec<OTPElement>;

    match database_loader::read_from_file(&cryptography::prompt_for_passwords("Password: ",8,false)){
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

    match database_loader::read_from_file(&cryptography::prompt_for_passwords("Password: ",8,false)){
        Ok(result) => elements = result,
        Err(e) => return Err(e),
    }

    match check_elements(index, &elements){
        Ok(()) => {},
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