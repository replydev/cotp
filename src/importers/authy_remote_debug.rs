/*
Import from JSON file exported from a script executed from remote debugging.
For more information see https://gist.github.com/gboudreau/94bb0c11a6209c82418d01a59d958c93
*/

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};
use serde::Deserialize;

const URL_INDEX: usize = 3;
const PARAMETERS_INDEX: usize = 1;
const DIGITS_DEFAULT_VALUE: u64 = 6;

#[derive(Deserialize)]
struct AuthyExportedJsonElement {
    name: String,
    secret: String,
    uri: String,
}

// Newtype pattern to bypass compiler check for impl From for Vec<AuthyExportedJsonElement>
// https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html
#[derive(Deserialize)]
pub struct AuthyExportedList(Vec<AuthyExportedJsonElement>);

impl AuthyExportedJsonElement {
    pub fn get_type(&self) -> String {
        let default_value = "totp";
        let args: Vec<&str> = self.uri.split('/').collect();
        String::from(*args.get(2).unwrap_or(&default_value))
    }

    pub fn get_digits(&self) -> u64 {
        let args: Vec<&str> = self.uri.split('/').collect();
        args.get(URL_INDEX)
            .and_then(|s| {
                let mut args: Vec<&str> = s.split('?').collect();
                if args.get(PARAMETERS_INDEX).is_some() {
                    Some(args.swap_remove(PARAMETERS_INDEX))
                } else {
                    None
                }
            })
            .and_then(|s| {
                let mut args: Vec<&str> =
                    s.split('&').filter(|s| s.starts_with("digits=")).collect();
                if !args.is_empty() {
                    Some(args.swap_remove(0))
                } else {
                    None
                }
            })
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(DIGITS_DEFAULT_VALUE)
    }

    pub fn get_issuer(&self) -> String {
        let default_value = "";
        let args: Vec<&str> = self.uri.split('/').collect();
        match args.get(3) {
            Some(s) => {
                let args: Vec<&str> = s.split('?').collect();
                let issuer = args.first().unwrap_or(&default_value);
                match urlencoding::decode(issuer) {
                    Ok(r) => r.into_owned(),
                    Err(_e) => (*issuer).to_string(),
                }
            }
            None => String::from(default_value),
        }
    }
}

impl From<AuthyExportedJsonElement> for OTPElement {
    fn from(input: AuthyExportedJsonElement) -> Self {
        let type_ = OTPType::from(input.get_type().as_str());
        let counter: Option<u64> = (type_ == OTPType::Hotp).then_some(0);
        let digits = input.get_digits();
        OTPElement {
            secret: input.secret.to_uppercase().replace('=', ""),
            issuer: input.get_issuer(),
            label: input.name,
            digits,
            type_,
            algorithm: OTPAlgorithm::Sha1,
            period: 30,
            counter,
            pin: None,
        }
    }
}

impl From<AuthyExportedList> for Vec<OTPElement> {
    fn from(exported_list: AuthyExportedList) -> Self {
        exported_list.0.into_iter().map(Into::into).collect()
    }
}
