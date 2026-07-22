use color_eyre::eyre::{ErrReport, eyre};
use derive_builder::Builder;
use std::{fs::File, io::Write, vec};

use crate::crypto::cryptography::{argon_derive_key, encrypt_string_with_key, gen_salt};
use crate::otp::otp_error::OtpError;
use crate::path::DATABASE_PATH;
use data_encoding::BASE32_NOPAD;
use qrcode::QrCode;
use qrcode::render::unicode;
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
        // The plaintext JSON contains every secret in the database: wipe it
        // from memory as soon as it has been encrypted
        let mut json = serde_json::to_string(&self)?;
        let encrypted = encrypt_string_with_key(&json, key, salt);
        json.zeroize();
        let encrypted = encrypted.unwrap();
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

static ALLOWED_DIGITS_RANGE: std::ops::RangeInclusive<u64> = 1..=10;

impl OTPElement {
    pub fn get_otpauth_uri(&self) -> String {
        let otp_type = self.type_.to_string().to_lowercase();
        let secret = &urlencoding::encode(self.secret.as_str());
        let label = get_label(&self.issuer, &self.label);
        let algorithm = self.algorithm.to_string().to_uppercase();
        let digits = self.digits;
        let period = self.period;
        let mut uri: String = format!(
            "otpauth://{otp_type}/{label}?secret={secret}&algorithm={algorithm}&digits={digits}&period={period}&lock=false"
        );

        if self.type_ == OTPType::Hotp {
            uri.push_str("&counter=");
            uri.push_str(self.counter.unwrap_or(0).to_string().as_str());
        }

        // Yandex / MOTP codes cannot be generated without their pin, so it
        // must survive an export / import round-trip
        if let Some(pin) = &self.pin {
            uri.push_str("&pin=");
            uri.push_str(&urlencoding::encode(pin));
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
        if !ALLOWED_DIGITS_RANGE.contains(&self.digits) {
            return Err(OtpError::InvalidDigits);
        }

        match self.type_ {
            OTPType::Totp => {
                let code = totp(&self.secret, self.algorithm, self.period)?;

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

    fn format_code(&self, value: u32) -> Result<String, OtpError> {
        format_code(self.digits, value)
    }
}

pub(crate) fn format_code(digits: u64, value: u32) -> Result<String, OtpError> {
    let exponential = 10_u64
        .checked_pow(digits as u32)
        .ok_or(OtpError::InvalidDigits)?;
    let s = (value as u64 % exponential).to_string();
    Ok("0".repeat((digits as usize).saturating_sub(s.chars().count())) + s.as_str())
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

        if self.period == Some(0) {
            return Err(eyre!("Period must be greater than zero",));
        }

        if self.digits == Some(0) {
            return Err(eyre!("Digits must be greater than zero",));
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
        assert_eq!(
            "otpauth://totp/IssuerText:LabelText?secret=xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g&algorithm=SHA1&digits=6&period=30&lock=false",
            otp_element.get_otpauth_uri().as_str()
        );
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
        assert_eq!(
            "otpauth://totp/:LabelText?secret=xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g&algorithm=SHA1&digits=6&period=30&lock=false",
            otp_element.get_otpauth_uri().as_str()
        );
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
        let invalid_digits_value = 11;

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
    fn test_10_digits_should_be_allowed() {
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
        assert!(result.is_ok());
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
    fn test_zero_period_is_rejected_by_builder() {
        let result = OTPElementBuilder::default()
            .secret("AA")
            .label("label")
            .issuer("")
            .period(0u64)
            .build();

        assert_eq!(
            "Period must be greater than zero",
            result.unwrap_err().to_string()
        );
    }

    #[test]
    fn test_zero_digits_is_rejected_by_builder() {
        let result = OTPElementBuilder::default()
            .secret("AA")
            .label("label")
            .issuer("")
            .digits(0u64)
            .build();

        assert_eq!(
            "Digits must be greater than zero",
            result.unwrap_err().to_string()
        );
    }

    #[test]
    fn test_zero_period_returns_error_instead_of_panicking() {
        let element = OTPElement {
            secret: "xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g".to_uppercase(),
            issuer: String::from("IssuerText"),
            label: String::from("LabelText"),
            digits: 6,
            type_: Totp,
            algorithm: Sha1,
            period: 0,
            counter: None,
            pin: None,
        };

        assert_eq!(Err(OtpError::InvalidPeriod), element.get_otp_code());
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

    fn assert_generation_relevant_fields_eq(expected: &OTPElement, actual: &OTPElement) {
        assert_eq!(expected.secret, actual.secret);
        assert_eq!(expected.type_, actual.type_);
        assert_eq!(expected.algorithm, actual.algorithm);
        assert_eq!(expected.digits, actual.digits);
        assert_eq!(expected.period, actual.period);
        assert_eq!(expected.counter, actual.counter);
        assert_eq!(expected.pin, actual.pin);
    }

    #[test]
    fn test_otp_uri_round_trip_totp() {
        let element = OTPElementBuilder::default()
            .secret("xr5gh44x7bprcqgrdtulafeevt5rxqlbh5wvked22re43dh2d4mapv5g")
            .issuer("IssuerText")
            .label("LabelText")
            .build()
            .unwrap();

        let round_tripped = OTPElement::from_otp_uri(&element.get_otpauth_uri()).unwrap();

        assert_generation_relevant_fields_eq(&element, &round_tripped);
    }

    #[test]
    fn test_otp_uri_round_trip_motp_preserves_lowercase_secret_and_pin() {
        let element = OTPElementBuilder::default()
            .secret("e3152afee62599c8")
            .type_(OTPType::Motp)
            .issuer("IssuerText")
            .label("LabelText")
            .period(10u64)
            .pin("1234".to_string())
            .build()
            .unwrap();

        let round_tripped = OTPElement::from_otp_uri(&element.get_otpauth_uri()).unwrap();

        assert_generation_relevant_fields_eq(&element, &round_tripped);
        // MOTP secrets are hex text hashed with MD5: uppercasing them changes
        // the generated codes
        assert_eq!("e3152afee62599c8", round_tripped.secret);
        assert_eq!(Some("1234".to_string()), round_tripped.pin);
    }

    #[test]
    fn test_otp_uri_round_trip_yandex_preserves_pin() {
        let element = OTPElementBuilder::default()
            .secret("6SB2IKNM6OBZPAVBVTOHDKS4FAAAAAAADFUTQMBTRY")
            .type_(OTPType::Yandex)
            .issuer("Yandex")
            .label("LabelText")
            .digits(8u64)
            .pin("5239".to_string())
            .build()
            .unwrap();

        let round_tripped = OTPElement::from_otp_uri(&element.get_otpauth_uri()).unwrap();

        assert_generation_relevant_fields_eq(&element, &round_tripped);
        assert_eq!(Some("5239".to_string()), round_tripped.pin);
        // Both must generate a code, not fail with a missing pin
        assert_eq!(element.get_otp_code(), round_tripped.get_otp_code());
    }

    #[test]
    fn test_from_otp_uri_rejects_invalid_base32_secret() {
        // Construction goes through OTPElementBuilder, so its validation
        // applies to URI imports too
        // "aaa" has an invalid BASE32 length and "1" is not in the alphabet
        let otp_uri = "otpauth://totp/Label?secret=aa1";

        assert!(OTPElement::from_otp_uri(otp_uri).is_err());
    }

    #[test]
    fn gh_issue_548_invalid_otp_uri_label_url_encoded() {
        // Arrange
        let otp_uri = "otpauth://totp/foo%3abar?issuer=foo&secret=JBSWY3DPEHPK3PXP";

        // Act
        let result = OTPElement::from_otp_uri(otp_uri);

        // Assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!("foo", actual.issuer.as_str());
        assert_eq!("bar", actual.label.as_str());
    }

    #[test]
    fn gh_issue_548_invalid_otp_uri_label_non_url_encoded() {
        // Arrange
        let otp_uri = "otpauth://totp/foo:bar?issuer=foo&secret=JBSWY3DPEHPK3PXP";

        // Act
        let result = OTPElement::from_otp_uri(otp_uri);

        // Assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!("foo", actual.issuer.as_str());
        assert_eq!("bar", actual.label.as_str());
    }
}
