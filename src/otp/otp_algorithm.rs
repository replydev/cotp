use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy, ValueEnum, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum OTPAlgorithm {
    Sha1,
    Sha256,
    Sha512,
    Md5,
}

impl fmt::Display for OTPAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
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
