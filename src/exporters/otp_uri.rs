use crate::otp::otp_element::OTPDatabase;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OtpUriList {
    pub items: Vec<String>,
}

impl From<OTPDatabase> for OtpUriList {
    fn from(value: OTPDatabase) -> Self {
        let items: Vec<String> = value
            .elements
            .into_iter()
            .map(|e| e.get_otpauth_uri())
            .collect();

        OtpUriList { items }
    }
}
