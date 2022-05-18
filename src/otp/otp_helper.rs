use crate::otp::otp_element::OTPElement;
use crate::otp::otp_maker::{hotp, totp};

pub fn get_otp_code(element: &OTPElement) -> Result<String, String> {
    match element.type_().to_uppercase().as_str() {
        "TOTP" => totp(
            &element.secret(),
            &element.algorithm().to_uppercase(),
            element.digits() as u32,
        ),
        "HOTP" => match element.counter() {
            Some(counter) => hotp(
                &element.secret(),
                &element.algorithm().to_uppercase(),
                element.digits() as u32,
                counter,
            ),
            None => Err(String::from(
                "The element is an HOTP code but the is no counter value.",
            )),
        },
        _ => unreachable!(),
    }
}
