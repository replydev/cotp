use crate::exporters::otp_uri::OtpUriList;
use crate::otp::from_otp_uri::FromOtpUri;
use crate::otp::otp_element::{OTPDatabase, OTPElement};

impl From<OtpUriList> for OTPDatabase {
    fn from(value: OtpUriList) -> Self {
        let input_length = value.items.len();
        let converted_elements = value
            .items
            .into_iter()
            .map(|e| OTPElement::from_otp_uri(e.as_str()))
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap())
            .collect();

        OTPDatabase::from(converted_elements)
    }
}
