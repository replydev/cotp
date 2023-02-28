use data_encoding::DecodeKind;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum OtpError {
    SecretEncoding(DecodeKind, usize), // Secret encoding error, of given kind at give position
    MissingPin,                        // Missing Pin for Yandex / MOTP Codes
    MissingCounter,                    // Missing counter for HOTP codes
    InvalidOffset,                     // Invalid offset
    InvalidDigest,                     // Invalid digest
}

impl Display for OtpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OtpError::SecretEncoding(decode_kind, position) => {
                f.write_str(format!("Decode error {decode_kind} at {position}").as_str())
            }
            OtpError::MissingPin => f.write_str("Missing pin value"),
            OtpError::MissingCounter => f.write_str("Missing counter value"),
            OtpError::InvalidDigest => f.write_str("Invalid digest"),
            OtpError::InvalidOffset => f.write_str("Invalid offset"),
        }
    }
}

impl std::error::Error for OtpError {}
