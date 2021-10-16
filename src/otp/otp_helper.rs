use crate::{cryptography, database_management};
use crate::otp::otp_element::OTPElement;
use crate::otp::otp_maker::make_totp;
use crate::utils::check_elements;
use zeroize::Zeroize;


pub fn read_codes() -> Result<Vec<OTPElement>, String> {
    let mut pw = cryptography::prompt_for_passwords("Password: ", 8, false);
    let result = match database_management::read_from_file(&pw) {
        Ok(result) => Ok(result),
        Err(e) => Err(e),
    };
    pw.zeroize();
    result
}

pub fn get_good_otp_code(element: &OTPElement) -> String {
    let otp = make_totp(
        &element.secret(), //we have replaced '=' in this method
        &element.algorithm().to_uppercase(), element.digits());

    "0".repeat(otp.chars().count() - element.digits() as usize) + otp.as_str()
}

pub fn print_element_info(mut index: usize) -> Result<(), String> {
    if index == 0 {
        return Err(String::from("Invalid index"));
    }
    index -= 1;

    let elements: Vec<OTPElement>;
    let mut pw = cryptography::prompt_for_passwords("Password: ", 8, false);
    match database_management::read_from_file(&pw) {
        Ok(result) => elements = result,
        Err(e) => return Err(e),
    }
    pw.zeroize();

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