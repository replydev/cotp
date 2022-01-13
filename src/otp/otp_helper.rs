use crate::{cryptography, database_management};
use crate::otp::otp_element::OTPElement;
use crate::otp::otp_maker::{hotp, totp};
use crate::utils::{check_elements, millis_before_next_step};
use copypasta_ext::prelude::ClipboardProvider;
use copypasta_ext::x11_fork::ClipboardContext;
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

pub fn get_otp_code(element: &OTPElement) -> Result<String,String> {
    match element.type_().to_uppercase().as_str() {
        "TOTP" => totp(&element.secret(), &element.algorithm().to_uppercase(), element.digits() as u32),
        "HOTP" => {
            match element.counter() {
                Some(counter) => hotp(&element.secret(), &element.algorithm().to_uppercase(), element.digits() as u32, counter),
                None => Err(String::from("The element is an HOTP code but the is no counter value.")),
            }
        },
        _ => unreachable!()
    }
}

pub fn print_elements_matching(issuer: Option<&str>, label: Option<&str>) -> Result<(), String> {
    let mut pw = cryptography::prompt_for_passwords("Password: ", 8, false);
    let elements = match database_management::read_from_file(&pw) {
        Ok(result) => result,
        Err(e) => return Err(e),
    };
    pw.zeroize();

    elements.iter().filter(|element| {
        (if let Some(i) = issuer {
            i.to_lowercase() == element.issuer().to_lowercase()
        }
        else {
            true
        }) && 
        (if let Some(l) = label {
            l.to_lowercase() == element.label().to_lowercase()
        }
        else {
            true
        })
    }).for_each(|element| {
        let otp_code = match get_otp_code(element) {
            Ok(code) => code,
            Err(e) => e,
        };
        println!("");
        println!("Issuer: {}", element.issuer());
        println!("Label: {}", element.label());
        println!("OTP Code: {} ({} seconds remaining)", otp_code, millis_before_next_step()/1000);
        if let Ok(mut ctx) = ClipboardContext::new(){
            match ctx.set_contents(otp_code.to_owned()) {
                Ok(_) => println!("Copied to clipboard"),
                Err(_) => println!("Cannot copy OTP Code to clipboard"),
            };
        }
        println!("");
    });
    Ok(())
}

pub fn print_element_info(mut index: usize) -> Result<(), String> {
    if index == 0 {
        return Err(String::from("Invalid index"));
    }
    index -= 1;

    let mut pw = cryptography::prompt_for_passwords("Password: ", 8, false);
    let elements = match database_management::read_from_file(&pw) {
        Ok(result) => result,
        Err(e) => return Err(e),
    };
    pw.zeroize();

    match check_elements(index, elements.as_slice()) {
        Ok(()) => {}
        Err(e) => {
            return Err(e);
        }
    }

    let chosen_element: &OTPElement = &elements[index];

    println!("Issuer: {}", chosen_element.issuer());
    println!("Label: {}", chosen_element.label());
    println!("Algorithm: {}", chosen_element.algorithm());
    println!("Type: {}",chosen_element.type_());
    println!("Digits: {}", chosen_element.digits());
    Ok(())
}