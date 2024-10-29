use crate::otp::otp_element::OTPDatabase;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OtpUriList {
    pub items: Vec<String>,
}

impl<'a> From<&'a OTPDatabase> for OtpUriList {
    fn from(value: &'a OTPDatabase) -> Self {
        let items: Vec<String> = value
            .elements
            .iter()
            .map(super::super::otp::otp_element::OTPElement::get_otpauth_uri)
            .collect();

        OtpUriList { items }
    }
}
