use std::{fs::File, io::Write, path::PathBuf, vec};

use crate::otp::otp_error::OtpError;
use crate::{
    crypto::cryptography::{argon_derive_key, encrypt_string_with_key, gen_salt},
    utils,
};
use data_encoding::BASE32_NOPAD;
use lazy_static::lazy_static;
use qrcode::render::unicode;
use qrcode::QrCode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

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

    pub fn save(&mut self, key: &Vec<u8>, salt: &[u8]) -> Result<(), String> {
        self.needs_modification = false;
        migrate(self)?;
        match self.overwrite_database_key(key, salt) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("{e:?}")),
        }
    }

    fn overwrite_database_key(&self, key: &Vec<u8>, salt: &[u8]) -> Result<(), std::io::Error> {
        let json: &str = &serde_json::to_string(&self)?;
        let encrypted = encrypt_string_with_key(json.to_string(), key, salt).unwrap();
        let mut file = File::create(utils::get_db_path())?;
        match serde_json::to_string(&encrypted) {
            Ok(content) => {
                file.write_all(content.as_bytes())?;
                file.sync_all()?;
                Ok(())
            }
            Err(e) => Err(std::io::Error::from(e)),
        }
    }

    pub fn save_with_pw(&mut self, password: &str) -> Result<(Vec<u8>, [u8; 16]), String> {
        let salt = gen_salt()?;
        let key = argon_derive_key(password.as_bytes(), &salt)?;
        self.save(&key, &salt)?;
        Ok((key, salt))
    }

    pub fn export(&self, path: PathBuf) -> Result<PathBuf, String> {
        if self.elements.is_empty() {
            return Err(String::from(
                "there are no elements in your database, type \"cotp -h\" to get help",
            ));
        }

        let exported_path = if path.is_dir() {
            path.join("exported.cotp")
        } else {
            path
        };

        match serde_json::to_string(self) {
            Ok(mut contents) => {
                if contents == "[]" {}
                let mut file = File::create(&exported_path).expect("Cannot create file");
                let contents_bytes = contents.as_bytes();
                file.write_all(contents_bytes)
                    .expect("Failed to write contents");
                contents.zeroize();
                Ok(exported_path)
            }
            Err(e) => Err(format!("{e:?}")),
        }
    }

    pub fn add_all(&mut self, mut elements: Vec<OTPElement>) {
        self.mark_modified();
        self.elements.append(&mut elements)
    }

    pub fn add_element(&mut self, element: OTPElement) {
        self.mark_modified();
        self.elements.push(element)
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
        })
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct OTPElement {
    pub secret: String,
    pub issuer: String,
    pub label: String,
    pub digits: u64,
    #[serde(rename = "type")]
    pub type_: OTPType,
    pub algorithm: OTPAlgorithm,
    pub period: u64,
    pub counter: Option<u64>,
    pub pin: Option<String>,
}

pub trait FromOtpUri: Sized {
    fn from_otp_uri(otp_uri: &str) -> Result<Self, String>;
}

impl FromOtpUri for OTPElement {
    fn from_otp_uri(otp_uri: &str) -> Result<Self, String> {
        lazy_static! {
            static ref TYPE_REGEX: Regex = Regex::new(r#"otpauth:[/][/]([a-zA-Z])[/]"#).unwrap();
            static ref NAME_REGEX: Regex = Regex::new(r#"[a-zA-Z][/](?:(.*):)(.+)\?"#).unwrap();
            static ref SECRET_REGEX: Regex = Regex::new(r#"[?&]secret=(.*?)(?:&|$)"#).unwrap();
            static ref ALGORITHM_REGEX: Regex =
                Regex::new(r#"[?&]algorithm=(.*?)(?:&|$)"#).unwrap();
            static ref DIGITS_REGEX: Regex = Regex::new(r#"[?&]digits=(\d*?)(?:&|$)"#).unwrap();
            static ref PERIOD_REGEX: Regex = Regex::new(r#"[?&]period=(\d*?)(?:&|$)"#).unwrap();
            static ref COUNTER_REGEX: Regex = Regex::new(r#"[?&]counter=(\d*?)(?:&|$)"#).unwrap();
        }

        let otp_type = get_match(&TYPE_REGEX, otp_uri)
            .map(|r| r.to_uppercase())
            .unwrap_or_else(|_| "TOTP".to_string());
        let (issuer, label) = NAME_REGEX
            .captures(otp_uri)
            .map(|c| {
                (
                    c.get(1).map(|v| v.as_str().to_string()),
                    c.get(2).map(|v| v.as_str().to_string()),
                )
            })
            .unwrap_or((None, None));

        if issuer.is_none() {
            return Err(String::from("Issuer not found in OTP Uri"));
        }

        let secret = get_match(&SECRET_REGEX, otp_uri)?.to_uppercase();
        let algorithm = get_match(&ALGORITHM_REGEX, otp_uri)
            .map(|r| r.to_uppercase())
            .unwrap_or_else(|_| "SHA1".to_string());
        let digits = get_match(&DIGITS_REGEX, otp_uri)
            .map(|r| r.parse::<u64>().unwrap())
            .unwrap_or(6);
        let period = get_match(&PERIOD_REGEX, otp_uri)
            .map(|r| r.parse::<u64>().unwrap())
            .unwrap_or(30);
        let counter = get_match(&COUNTER_REGEX, otp_uri)
            .map(|r| Some(r.parse::<u64>().unwrap()))
            .unwrap_or(None);

        Ok(OTPElement {
            secret,
            issuer: issuer.unwrap(), // Safe to wrap due to upper check
            label: label.unwrap_or_default(),
            digits,
            type_: OTPType::from(otp_type.as_str()),
            algorithm: OTPAlgorithm::from(algorithm.as_str()),
            period,
            counter,
            pin: None,
        })
    }
}

fn get_match(regex: &Regex, value: &str) -> Result<String, String> {
    let optional_value = regex.captures(value);
    if optional_value.is_none() {
        return Err(String::from("No match found"));
    }
    let match_str = optional_value.unwrap().get(1).unwrap();
    Ok(match_str.as_str().to_owned())
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

                Ok(self.format_code(code))
            }
            OTPType::Hotp => match self.counter {
                Some(counter) => {
                    let code = hotp(&self.secret, self.algorithm, counter)?;

                    Ok(self.format_code(code))
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
                Some(pin) => motp(
                    &self.secret,
                    pin.as_str(),
                    self.period,
                    self.digits as usize,
                ),
                None => Err(OtpError::MissingPin),
            },
        }
    }

    pub fn format_code(&self, value: u32) -> String {
        // Get the formatted code
        let s = (value % 10_u32.pow(self.digits as u32)).to_string();
        "0".repeat(self.digits as usize - s.chars().count()) + s.as_str()
    }

    pub fn valid_secret(&self) -> bool {
        match self.type_ {
            OTPType::Motp => hex::decode(&self.secret).is_ok(),
            _ => BASE32_NOPAD.decode(self.secret.as_bytes()).is_ok(),
        }
    }
}

fn get_label(issuer: &str, label: &str) -> String {
    let encoded_label = urlencoding::encode(label);
    if !issuer.is_empty() {
        let encoded_issuer = urlencoding::encode(issuer);
        return format!("{encoded_issuer}:{encoded_label}");
    }
    encoded_label.to_string()
}

#[cfg(test)]
mod test {
    use crate::otp::otp_element::OTPAlgorithm::Sha1;
    use crate::otp::otp_element::OTPElement;
    use crate::otp::otp_element::OTPType::Totp;

    use super::FromOtpUri;

    #[test]
    fn test_serialization_otp_uri() {
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
        assert_eq!(otp_element.get_otpauth_uri().as_str(), "otpauth://totp/IssuerText:LabelText?secret=xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g&algorithm=SHA1&digits=6&period=30&lock=false");
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

        assert_eq!(expected, OTPElement::from_otp_uri(otp_uri).unwrap())
    }
}
