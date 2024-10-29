use color_eyre::eyre::{eyre, ErrReport};
use derive_builder::Builder;
use std::{fs::File, io::Write, vec};

use crate::crypto::cryptography::{argon_derive_key, encrypt_string_with_key, gen_salt};
use crate::otp::otp_error::OtpError;
use crate::path::DATABASE_PATH;
use data_encoding::BASE32_NOPAD;
use qrcode::render::unicode;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::{
    algorithms::{
        hotp_maker::hotp, motp_maker::motp, steam_otp_maker::steam, totp_maker::totp,
        yandex_otp_maker::yandex,
    },
    migrations::migrate,
    otp_algorithm::OTPAlgorithm,
    otp_type::OTPType,
};

pub const CURRENT_DATABASE_VERSION: u16 = 2;

#[derive(Serialize, Deserialize, PartialEq, Hash)]
pub struct OTPDatabase {
    pub(crate) version: u16,
    pub(crate) elements: Vec<OTPElement>,
    #[serde(skip)]
    pub(crate) needs_modification: bool,
}

impl From<Vec<OTPElement>> for OTPDatabase {
    /// Build the first version of `OTPDatabase`, which was only a vector of `OTPElements`
    fn from(val: Vec<OTPElement>) -> Self {
        OTPDatabase {
            version: 1,
            elements: val,
            needs_modification: true,
        }
    }
}

impl Default for OTPDatabase {
    fn default() -> Self {
        Self {
            version: CURRENT_DATABASE_VERSION,
            elements: vec![],
            needs_modification: false,
        }
    }
}

impl OTPDatabase {
    pub fn is_modified(&self) -> bool {
        self.needs_modification
    }

    pub fn save(&mut self, key: &Vec<u8>, salt: &[u8]) -> color_eyre::Result<()> {
        self.needs_modification = false;
        migrate(self)?;
        match self.overwrite_database_key(key, salt) {
            Ok(()) => Ok(()),
            Err(e) => Err(ErrReport::from(e)),
        }
    }

    fn overwrite_database_key(&self, key: &Vec<u8>, salt: &[u8]) -> Result<(), std::io::Error> {
        let json: &str = &serde_json::to_string(&self)?;
        let encrypted = encrypt_string_with_key(json, key, salt).unwrap();
        let mut file = File::create(DATABASE_PATH.get().unwrap())?;
        match serde_json::to_string(&encrypted) {
            Ok(content) => {
                file.write_all(content.as_bytes())?;
                file.sync_all()?;
                Ok(())
            }
            Err(e) => Err(std::io::Error::from(e)),
        }
    }

    pub fn save_with_pw(&mut self, password: &str) -> color_eyre::Result<(Vec<u8>, [u8; 16])> {
        let salt = gen_salt()?;
        let key = argon_derive_key(password.as_bytes(), &salt)?;
        self.save(&key, &salt)?;
        Ok((key, salt))
    }

    pub fn add_all(&mut self, mut elements: Vec<OTPElement>) {
        self.mark_modified();
        self.elements.append(&mut elements);
    }

    pub fn add_element(&mut self, element: OTPElement) {
        self.mark_modified();
        self.elements.push(element);
    }

    pub fn mark_modified(&mut self) {
        self.needs_modification = true;
    }

    pub fn delete_element(&mut self, index: usize) {
        self.mark_modified();
        self.elements.remove(index);
    }

    pub fn elements_ref(&self) -> &[OTPElement] {
        &self.elements
    }

    pub fn get_element(&self, i: usize) -> Option<&OTPElement> {
        self.elements.get(i)
    }

    pub fn mut_element(&mut self, i: usize) -> Option<&mut OTPElement> {
        self.elements.get_mut(i)
    }

    pub fn sort(&mut self) {
        self.elements.sort_unstable_by(|c1, c2| {
            c1.issuer
                .to_ascii_lowercase()
                .cmp(&c2.issuer.to_ascii_lowercase())
        });
    }
}

#[derive(
    Serialize, Deserialize, Builder, Clone, PartialEq, Eq, Debug, Hash, Zeroize, ZeroizeOnDrop,
)]
#[builder(
    setter(into),
    build_fn(validate = "Self::validate", error = "ErrReport")
)]
pub struct OTPElement {
    #[builder(setter(custom))]
    pub secret: String,
    pub issuer: String,
    pub label: String,
    #[builder(default = "6")]
    pub digits: u64,
    #[serde(rename = "type")]
    #[builder(setter(custom), default)]
    pub type_: OTPType,
    #[builder(default)]
    pub algorithm: OTPAlgorithm,
    #[builder(default = "30")]
    pub period: u64,
    #[builder(setter(into), default)]
    pub counter: Option<u64>,
    #[builder(setter(into), default)]
    pub pin: Option<String>,
}

impl OTPElement {
    pub fn get_otpauth_uri(&self) -> String {
        let otp_type = self.type_.to_string().to_lowercase();
        let secret = &urlencoding::encode(self.secret.as_str());
        let label = get_label(&self.issuer, &self.label);
        let algorithm = self.algorithm.to_string().to_uppercase();
        let digits = self.digits;
        let period = self.period;
        let mut uri: String = format!("otpauth://{otp_type}/{label}?secret={secret}&algorithm={algorithm}&digits={digits}&period={period}&lock=false");

        if self.type_ == OTPType::Hotp {
            uri.push_str("&counter=");
            uri.push_str(self.counter.unwrap_or(0).to_string().as_str());
        }
        uri
    }

    pub fn get_qrcode(&self) -> String {
        QrCode::new(self.get_otpauth_uri())
            .unwrap()
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build()
    }

    pub fn get_otp_code(&self) -> Result<String, OtpError> {
        match self.type_ {
            OTPType::Totp => {
                let code = totp(&self.secret, self.algorithm)?;

                Ok(self.format_code(code)?)
            }
            OTPType::Hotp => match self.counter {
                Some(counter) => {
                    let code = hotp(&self.secret, self.algorithm, counter)?;

                    Ok(self.format_code(code)?)
                }
                None => Err(OtpError::MissingCounter),
            },
            OTPType::Steam => steam(&self.secret, self.algorithm, self.digits as usize),
            OTPType::Yandex => match &self.pin {
                Some(pin) => yandex(
                    &self.secret,
                    pin.as_str(),
                    self.period,
                    self.digits as usize,
                    self.algorithm,
                ),
                None => Err(OtpError::MissingPin),
            },
            OTPType::Motp => match &self.pin {
                Some(pin) => Ok(motp(
                    &self.secret,
                    pin.as_str(),
                    self.period,
                    self.digits as usize,
                )),
                None => Err(OtpError::MissingPin),
            },
        }
    }

    fn format_code(&self, value: u32) -> Result<String, OtpError> {
        // Get the formatted code
        let exponential = 10_u32
            .checked_pow(self.digits as u32)
            .ok_or(OtpError::InvalidDigits)?;
        let s = (value % exponential).to_string();
        Ok("0".repeat(self.digits as usize - s.chars().count()) + s.as_str())
    }
}

impl OTPElementBuilder {
    /// Makes the secret insertion case insensitive
    pub fn secret<VALUE: Into<String>>(&mut self, value: VALUE) -> &mut Self {
        self.secret = Some(value.into().to_uppercase());
        self
    }

    /// Makes the secret insertion case insensitive
    pub fn type_<VALUE: Into<OTPType>>(&mut self, value: VALUE) -> &mut Self {
        let otp_type: OTPType = value.into();

        if otp_type == OTPType::Motp {
            // Motp codes must be lowercase
            self.secret = self.secret.as_ref().map(|s| s.to_lowercase());
        } else {
            // Base32 codes must be uppercase
            self.secret = self.secret.as_ref().map(|s| s.to_uppercase());
        }

        self.type_ = Some(otp_type);
        self
    }

    /// Check if the `OTPElement` is valid
    fn validate(&self) -> Result<(), ErrReport> {
        if self.secret.is_none() {
            return Err(eyre!("Secret must be set",));
        }

        if self.secret.as_ref().unwrap().is_empty() {
            return Err(eyre!("Secret must not be empty",));
        }

        // Validate secret encoding
        match self.type_.unwrap_or_default() {
            OTPType::Motp => hex::decode(self.secret.as_ref().unwrap())
                .map(|_| {})
                .map_err(|e| eyre!("Invalid hex secret: {e}")),
            _ => BASE32_NOPAD
                .decode(self.secret.as_ref().unwrap().as_bytes())
                .map(|_| {})
                .map_err(|e| eyre!("Invalid BASE32 secret: {e}")),
        }
    }
}

fn get_label(issuer: &str, label: &str) -> String {
    let encoded_label = urlencoding::encode(label);
    let encoded_issuer = urlencoding::encode(issuer);
    format!("{encoded_issuer}:{encoded_label}")
}

#[cfg(test)]
mod test {
    use crate::otp::otp_element::OTPAlgorithm::Sha1;
    use crate::otp::otp_element::OTPType::Totp;
    use crate::otp::otp_element::{OTPElement, OTPElementBuilder};

    use crate::otp::from_otp_uri::FromOtpUri;
    use crate::otp::otp_error::OtpError;
    use crate::otp::otp_type::OTPType;

    #[test]
    fn test_serialization_otp_uri_full_element() {
        let otp_element = OTPElement {
            secret: String::from("xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g"),
            issuer: String::from("IssuerText"),
            label: String::from("LabelText"),
            digits: 6,
            type_: Totp,
            algorithm: Sha1,
            period: 30,
            counter: None,
            pin: None,
        };
        assert_eq!("otpauth://totp/IssuerText:LabelText?secret=xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g&algorithm=SHA1&digits=6&period=30&lock=false",otp_element.get_otpauth_uri().as_str());
    }

    #[test]
    fn test_serialization_otp_uri_no_issuer() {
        let otp_element = OTPElement {
            secret: String::from("xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g"),
            issuer: String::new(),
            label: String::from("LabelText"),
            digits: 6,
            type_: Totp,
            algorithm: Sha1,
            period: 30,
            counter: None,
            pin: None,
        };
        assert_eq!("otpauth://totp/:LabelText?secret=xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g&algorithm=SHA1&digits=6&period=30&lock=false",otp_element.get_otpauth_uri().as_str());
    }

    #[test]
    fn test_deserialization_otp_uri() {
        let expected = OTPElement {
            secret: "xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g".to_uppercase(),
            issuer: String::from("IssuerText"),
            label: String::from("LabelText"),
            digits: 6,
            type_: Totp,
            algorithm: Sha1,
            period: 30,
            counter: None,
            pin: None,
        };
        let otp_uri = "otpauth://totp/IssuerText:LabelText?secret=xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g&algorithm=SHA1&digits=6&period=30&lock=false";

        assert_eq!(expected, OTPElement::from_otp_uri(otp_uri).unwrap());
    }

    #[test]
    fn test_deserialization_with_issuer_parameter() {
        let otp_uri = "otpauth://totp/2Ponies%40Github%20No.1?secret=JBSWY3DPEHPK3PXP&algorithm=SHA1&digits=6&period=30&lock=false&issuer=test";
        assert!(OTPElement::from_otp_uri(otp_uri).is_ok());
    }

    #[test]
    fn test_invalid_digits_should_not_overflow() {
        // Arrange
        let invalid_digits_value = 10;

        let element = OTPElement {
            secret: "xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g".to_uppercase(),
            issuer: String::from("IssuerText"),
            label: String::from("LabelText"),
            digits: invalid_digits_value,
            type_: Totp,
            algorithm: Sha1,
            period: 30,
            counter: None,
            pin: None,
        };

        // Act
        let result = element.get_otp_code();

        // Assert
        assert_eq!(Err(OtpError::InvalidDigits), result);
    }

    #[test]
    fn test_lowercase_secret() {
        // Arrange / Act
        let result = OTPElementBuilder::default()
            .secret("aa")
            .label("label")
            .issuer("")
            .build();

        // Assert
        assert_eq!("AA", result.unwrap().secret);
    }

    #[test]
    fn test_invalid_secret_base32() {
        let result = OTPElementBuilder::default()
            .secret("aaa")
            .label("label")
            .issuer("")
            .build();

        assert_eq!(
            "Invalid BASE32 secret: invalid length at 2",
            result.unwrap_err().to_string()
        );
    }

    #[test]
    fn valid_hex_secret() {
        let result = OTPElementBuilder::default()
            .secret("aAAf")
            .label("label")
            .issuer("")
            .type_(OTPType::Motp)
            .build();

        assert_eq!("aaaf", result.unwrap().secret);
    }

    #[test]
    fn invalid_secret_hex() {
        let result = OTPElementBuilder::default()
            .secret("aaa")
            .label("label")
            .issuer("")
            .type_(OTPType::Motp)
            .build();

        assert_eq!(
            "Invalid hex secret: Odd number of digits",
            result.unwrap_err().to_string()
        );
    }
}
