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

impl From<&str> for OTPAlgorithm {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "SHA256" => Self::Sha256,
            "SHA512" => Self::Sha512,
            "MD5" => Self::Md5,
            _ => Self::Sha1,
        }
    }
}

impl Zeroize for OTPAlgorithm {
    fn zeroize(&mut self) {
        *self = OTPAlgorithm::Sha1;
    }
}
