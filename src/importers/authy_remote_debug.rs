/*
Import from JSON file exported from a script executed from remote debugging.
For more information see https://gist.github.com/gboudreau/94bb0c11a6209c82418d01a59d958c93
*/

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};
use serde::Deserialize;

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
        let default_value = 6;
        let args: Vec<&str> = self.uri.split('/').collect();
        match args.get(3) {
            Some(s) => {
                let args: Vec<&str> = s.split('?').collect();
                match args.get(1) {
                    Some(s) => {
                        let args: Vec<&str> =
                            s.split('&').filter(|s| s.starts_with("digits=")).collect();
                        match args.first() {
                            Some(s) => s.parse::<u64>().unwrap_or(default_value),
                            None => default_value,
                        }
                    }
                    None => default_value,
                }
            }
            None => default_value,
        }
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
                    Err(_e) => issuer.to_string(),
                }
            }
            None => String::from(default_value),
        }
    }
}

impl From<AuthyExportedJsonElement> for OTPElement {
    fn from(input: AuthyExportedJsonElement) -> Self {
        let type_ = OTPType::from(input.get_type().as_str());
        let counter: Option<u64> = if type_ == OTPType::Hotp {
            Some(0)
        } else {
            None
        };
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
        exported_list.0.into_iter().map(|e| e.into()).collect()
    }
}
