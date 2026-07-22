use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy, ValueEnum, Hash, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum OTPType {
    #[serde(alias = "totp")]
    #[serde(alias = "TOTP")]
    #[default]
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
        let to_string = match self {
            OTPType::Totp => "TOTP",
            OTPType::Hotp => "HOTP",
            OTPType::Steam => "STEAM",
            OTPType::Yandex => "YANDEX",
            OTPType::Motp => "MOTP",
        };
        write!(f, "{to_string}")
    }
}

impl TryFrom<&str> for OTPType {
    type Error = eyre::Report;

    /// Parses an OTP type name case-insensitively, rejecting unknown values
    /// instead of silently defaulting to TOTP (which would generate wrong
    /// codes for entries of a different, unsupported type).
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_uppercase().as_str() {
            "TOTP" => Ok(Self::Totp),
            "HOTP" => Ok(Self::Hotp),
            "STEAM" => Ok(Self::Steam),
            "YANDEX" => Ok(Self::Yandex),
            "MOTP" => Ok(Self::Motp),
            _ => Err(eyre::eyre!(
                "Unknown OTP type: {s:?} (expected one of TOTP, HOTP, STEAM, YANDEX, MOTP)"
            )),
        }
    }
}

impl Zeroize for OTPType {
    fn zeroize(&mut self) {
        *self = OTPType::Totp;
    }
}

#[cfg(test)]
mod tests {
    use super::OTPType;

    #[test]
    fn known_types_parse_case_insensitively() {
        assert_eq!(OTPType::Totp, OTPType::try_from("totp").unwrap());
        assert_eq!(OTPType::Totp, OTPType::try_from("TOTP").unwrap());
        assert_eq!(OTPType::Hotp, OTPType::try_from("HoTp").unwrap());
        assert_eq!(OTPType::Steam, OTPType::try_from("steam").unwrap());
        assert_eq!(OTPType::Yandex, OTPType::try_from("YANDEX").unwrap());
        assert_eq!(OTPType::Motp, OTPType::try_from("Motp").unwrap());
    }

    #[test]
    fn unknown_type_is_an_error_instead_of_defaulting_to_totp() {
        let error = OTPType::try_from("otp-2000").unwrap_err();
        assert!(error.to_string().contains("Unknown OTP type"));
        assert!(error.to_string().contains("otp-2000"));
    }
}
