use prettytable::{Table, row, cell,format};
use serde_json;
use serde::{Deserialize, Serialize};
use crate::{cryptograpy, database_loader};
use crate::otp::otp_element::OTPElement;
use crate::otp::otp_maker::make_totp;
use crate::utils::check_elements;
use crate::print_settings::PrintSettings;


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

pub fn read_codes() -> Result<Vec<OTPElement>,String>{
    match database_loader::read_from_file(&cryptograpy::prompt_for_passwords("Password: ", 8,false)){
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
    temp_print.set_max_id(index.len());
    temp_print.set_max_issuer(issuer.len());
    temp_print.set_max_label(label.len());
    temp_print.set_max_code(code.len());

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