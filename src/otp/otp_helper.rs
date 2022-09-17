use crate::otp::otp_element::OTPElement;
use crate::otp::otp_maker::{hotp, totp};

use super::steam_otp_maker::steam;

pub fn get_otp_code(element: &OTPElement) -> Result<String, String> {
    match element.type_().to_uppercase().as_str() {
        "TOTP" => {
            let code = totp(&element.secret(), &element.algorithm().to_uppercase())?;

            Ok(format_code(code, element.digits() as usize))
        }
        "HOTP" => match element.counter() {
            Some(counter) => {
                let code = hotp(
                    &element.secret(),
                    &element.algorithm().to_uppercase(),
                    counter,
                )?;

                Ok(format_code(code, element.digits() as usize))
            }
            None => Err(String::from(
                "The element is an HOTP code but the is no counter value.",
            )),
        },
        "STEAM" => steam(element),
        _ => unreachable!(),
    }
}

pub fn format_code(value: u32, digits: usize) -> String {
    // Get the formatted code
    let s = (value % 10_u32.pow(digits as u32)).to_string();
    "0".repeat(digits - s.chars().count()) + s.as_str()
}
