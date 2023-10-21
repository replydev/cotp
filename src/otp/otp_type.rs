use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy, ValueEnum, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum OTPType {
    #[serde(alias = "totp")]
    #[serde(alias = "TOTP")]
    Totp,
    #[serde(alias = "hotp")]
    #[serde(alias = "HOTP")]
    Hotp,
    #[serde(alias = "steam")]
    #[serde(alias = "STEAM")]
    Steam,
    #[serde(alias = "yandex")]
    #[serde(alias = "YANDEX")]
    Yandex,
    #[serde(alias = "motp")]
    #[serde(alias = "MOTP")]
    Motp,
}

impl fmt::Display for OTPType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<&str> for OTPType {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "HOTP" => Self::Hotp,
            "STEAM" => Self::Steam,
            "YANDEX" => Self::Yandex,
            "MOTP" => Self::Motp,
            _ => Self::Totp,
        }
    }
}

impl Zeroize for OTPType {
    fn zeroize(&mut self) {
        *self = OTPType::Totp
    }
}
