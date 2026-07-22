use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy, ValueEnum, Hash, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum OTPAlgorithm {
    #[default]
    Sha1,
    Sha256,
    Sha512,
    Md5,
}

impl fmt::Display for OTPAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_string = match self {
            OTPAlgorithm::Sha1 => "SHA1",
            OTPAlgorithm::Sha256 => "SHA256",
            OTPAlgorithm::Sha512 => "SHA512",
            OTPAlgorithm::Md5 => "MD5",
        };
        write!(f, "{to_string}")
    }
}

impl TryFrom<&str> for OTPAlgorithm {
    type Error = eyre::Report;

    /// Parses an OTP algorithm name case-insensitively, rejecting unknown
    /// values instead of silently defaulting to SHA1 (which would generate
    /// wrong codes for entries using a different, unsupported algorithm).
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_uppercase().as_str() {
            "SHA1" => Ok(Self::Sha1),
            "SHA256" => Ok(Self::Sha256),
            "SHA512" => Ok(Self::Sha512),
            "MD5" => Ok(Self::Md5),
            _ => Err(eyre::eyre!(
                "Unknown OTP algorithm: {s:?} (expected one of SHA1, SHA256, SHA512, MD5)"
            )),
        }
    }
}

impl Zeroize for OTPAlgorithm {
    fn zeroize(&mut self) {
        *self = OTPAlgorithm::Sha1;
    }
}

#[cfg(test)]
mod tests {
    use super::OTPAlgorithm;

    #[test]
    fn known_algorithms_parse_case_insensitively() {
        assert_eq!(OTPAlgorithm::Sha1, OTPAlgorithm::try_from("sha1").unwrap());
        assert_eq!(OTPAlgorithm::Sha1, OTPAlgorithm::try_from("SHA1").unwrap());
        assert_eq!(
            OTPAlgorithm::Sha256,
            OTPAlgorithm::try_from("Sha256").unwrap()
        );
        assert_eq!(
            OTPAlgorithm::Sha512,
            OTPAlgorithm::try_from("sha512").unwrap()
        );
        assert_eq!(OTPAlgorithm::Md5, OTPAlgorithm::try_from("md5").unwrap());
    }

    #[test]
    fn unknown_algorithm_is_an_error_instead_of_defaulting_to_sha1() {
        let error = OTPAlgorithm::try_from("crc32").unwrap_err();
        assert!(error.to_string().contains("Unknown OTP algorithm"));
        assert!(error.to_string().contains("crc32"));
    }
}
