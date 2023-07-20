use crate::otp::otp_element::{OTPDatabase, OTPElement};

use super::Exporter;

type AndOtpDatabase = Vec<OTPElement>;
impl Exporter<AndOtpDatabase> for OTPDatabase {
    fn export(self: Self) -> AndOtpDatabase {
        self.elements
    }
}
