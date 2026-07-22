use crate::otp::otp_element::{OTPDatabase, OTPElement};

/// andOTP backups are plain JSON arrays of OTP elements
type AndOtpDatabase = Vec<OTPElement>;
impl From<OTPDatabase> for AndOtpDatabase {
    fn from(value: OTPDatabase) -> Self {
        value.into_elements()
    }
}

impl<'a> From<&'a OTPDatabase> for &'a [OTPElement] {
    fn from(value: &'a OTPDatabase) -> Self {
        value.elements_ref()
    }
}
