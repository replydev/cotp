use crate::otp::otp_element::{OTPDatabase, OTPElement};

type AndOtpDatabase = Vec<OTPElement>;
impl From<OTPDatabase> for AndOtpDatabase {
    fn from(value: OTPDatabase) -> Self {
        value.elements
    }
}

impl<'a> From<&'a OTPDatabase> for &'a AndOtpDatabase {
    fn from(value: &'a OTPDatabase) -> Self {
        &value.elements
    }
}
