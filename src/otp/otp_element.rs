use std::{fmt, vec};

use crate::otp::otp_element::OTPType::*;
use qrcode::render::unicode;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};

use super::{
    motp_maker::motp,
    otp_maker::{hotp, totp},
    steam_otp_maker::steam,
    yandex_otp_maker::yandex,
};

pub const CURRENT_DATABASE_VERSION: u16 = 2;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum OTPAlgorithm {
    OTPSha1,
    OTPSha256,
    OTPSha512,
    OTPMd5,
}

impl fmt::Display for OTPAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&str> for OTPAlgorithm {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "SHA256" => Self::OTPSha256,
            "SHA512" => Self::OTPSha512,
            "MD5" => OTPAlgorithm::OTPMd5,
            _ => Self::OTPSha1,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum OTPType {
    Totp,
    Hotp,
    Steam,
    Yandex,
    Motp,
}

impl fmt::Display for OTPType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&str> for OTPType {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "HOTP" => Hotp,
            "STEAM" => Steam,
            "Yandex" => Yandex,
            "Motp" => Motp,
            _ => Totp,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct OTPDatabase {
    version: u16,
    elements: Vec<OTPElement>,
}

impl OTPDatabase {
    pub fn new(version: u16, elements: Vec<OTPElement>) -> OTPDatabase {
        OTPDatabase { version, elements }
    }

    pub fn add_element(&mut self, element: OTPElement) {
        self.elements.push(element)
    }

    pub fn edit_element(&mut self, index: usize, element: OTPElement) {
        self.elements[index] = element;
    }

    pub fn delete_element(&mut self, index: usize) {
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
        self.elements.sort_by(|c1, c2| c1.issuer.cmp(&c2.issuer))
    }
}

#[derive(Serialize, Deserialize, Clone)]
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

impl OTPElement {
    pub fn get_otpauth_uri(&self) -> String {
        let mut uri: String = String::from("otpauth://");
        uri.push_str(self.type_.to_string().to_lowercase().as_str());
        uri.push('/');
        //self.type_.to_lowercase() + String::from("/");
        if self.issuer.chars().count() > 0 {
            uri.push_str(&urlencoding::encode(self.issuer.as_str()));
            uri.push(':');
        }
        uri.push_str(&urlencoding::encode(self.label.as_str()));

        uri.push_str("?secret=");
        uri.push_str(self.secret.as_str());
        uri.push_str("&algorithm=");
        uri.push_str(self.algorithm.to_string().to_uppercase().as_str());
        uri.push_str("&digits=");
        uri.push_str(self.digits.to_string().as_str());
        uri.push_str("&period=");
        uri.push_str(self.period.to_string().as_str());
        uri.push_str("&lock=false");
        //uri.push_str("?secret=" + self.secret());
        if self.type_ == Hotp {
            uri.push_str("&counter=");
            uri.push_str(self.counter.unwrap_or(0).to_string().as_str());
        }
        uri
    }

    pub fn get_qrcode(&self) -> String {
        QrCode::new(&self.get_otpauth_uri())
            .unwrap()
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build()
    }

    pub fn get_otp_code(&self) -> Result<String, String> {
        match self.type_ {
            Totp => {
                let code = totp(&self.secret, self.algorithm)?;

                Ok(self.format_code(code))
            }
            Hotp => match self.counter {
                Some(counter) => {
                    let code = hotp(&self.secret, self.algorithm, counter)?;

                    Ok(self.format_code(code))
                }
                None => Err(String::from(
                    "The element is an HOTP code but there is no counter value.",
                )),
            },
            Steam => steam(&self.secret, self.algorithm, self.digits as usize),
            Yandex => match &self.pin {
                Some(pin) => yandex(
                    &self.secret,
                    pin.as_str(),
                    self.period,
                    self.digits as usize,
                    self.algorithm,
                ),
                None => Err(String::from(
                    "This element is a Yandex code but there is not pin value",
                )),
            },
            Motp => match &self.pin {
                Some(pin) => motp(
                    &self.secret,
                    pin.as_str(),
                    self.period as u64,
                    self.digits as usize,
                ),
                None => Err(String::from(
                    "This element is an MOTP code but the is not pin value",
                )),
            },
            _ => unreachable!(),
        }
    }

    pub fn format_code(&self, value: u32) -> String {
        // Get the formatted code
        let s = (value % 10_u32.pow(self.digits as u32)).to_string();
        "0".repeat(self.digits as usize - s.chars().count()) + s.as_str()
    }
}

#[cfg(test)]
mod test {
    use crate::otp::otp_element::OTPAlgorithm::OTPSha1;
    use crate::otp::otp_element::OTPElement;
    use crate::otp::otp_element::OTPType::Totp;

    #[test]
    fn test_otpauth_uri() {
        let otp_element = OTPElement {
            secret: String::from("xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g"),
            issuer: String::from("IssuerText"),
            label: String::from("LabelText"),
            digits: 6,
            type_: Totp,
            algorithm: OTPSha1,
            period: 30,
            counter: None,
            pin: None,
        };
        assert_eq!(otp_element.get_otpauth_uri().as_str(), "otpauth://totp/IssuerText:LabelText?secret=xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g&algorithm=SHA1&digits=6&period=30&lock=false");
    }
}
