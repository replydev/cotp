use color_eyre::eyre::{Result, eyre};
use url::Url;

use super::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

pub trait FromOtpUri: Sized {
    fn from_otp_uri(otp_uri: &str) -> Result<Self>;
}

impl FromOtpUri for OTPElement {
    fn from_otp_uri(otp_uri: &str) -> Result<Self> {
        let uri = Url::parse(otp_uri).map_err(|_| eyre!("Invalid OTP URI"))?;
        if uri.scheme() != "otpauth" {
            return Err(eyre!("Expected otpauth:// URI"));
        }
        let type_ = match uri.host_str().unwrap_or("").to_ascii_lowercase().as_str() {
            "totp" => OTPType::Totp,
            "hotp" => OTPType::Hotp,
            "steam" => OTPType::Steam,
            "yandex" => OTPType::Yandex,
            "motp" => OTPType::Motp,
            _ => return Err(eyre!("Unsupported OTP type")),
        };
        let mut params = std::collections::HashMap::new();
        for (key, value) in uri.query_pairs() {
            if params
                .insert(key.into_owned(), value.into_owned())
                .is_some()
            {
                return Err(eyre!("Duplicate OTP URI parameter"));
            }
        }
        let label = urlencoding::decode(uri.path().trim_start_matches('/'))
            .map_err(|_| eyre!("Invalid OTP label encoding"))?;
        let (issuer, label) = label
            .split_once(':')
            .map(|(i, l)| (i.to_owned(), l.to_owned()))
            .unwrap_or_else(|| {
                (
                    params.get("issuer").cloned().unwrap_or_default(),
                    label.into_owned(),
                )
            });
        let secret = params
            .get("secret")
            .cloned()
            .ok_or_else(|| eyre!("Secret not found in OTP URI"))?;
        let algorithm = match params
            .get("algorithm")
            .map(|s| s.to_ascii_uppercase())
            .as_deref()
            .unwrap_or("SHA1")
        {
            "SHA1" => OTPAlgorithm::Sha1,
            "SHA256" => OTPAlgorithm::Sha256,
            "SHA512" => OTPAlgorithm::Sha512,
            "MD5" => OTPAlgorithm::Md5,
            _ => return Err(eyre!("Unsupported OTP algorithm")),
        };
        let number = |name: &str, default| -> Result<u64> {
            params
                .get(name)
                .map(|s| {
                    s.parse::<u64>()
                        .map_err(|_| eyre!("Invalid numeric OTP parameter: {name}"))
                })
                .unwrap_or(Ok(default))
        };
        let counter = if params.contains_key("counter") {
            Some(number("counter", 0)?)
        } else {
            None
        };
        let mut element = OTPElement {
            id: String::new(),
            alias: None,
            tags: Vec::new(),
            favorite: false,
            order: None,
            secret,
            issuer,
            label,
            digits: number("digits", 6)?,
            type_,
            algorithm,
            period: number("period", 30)?,
            counter,
            pin: params.get("pin").cloned(),
        };
        crate::importers::enrollment::validate(&mut element)?;
        Ok(element)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_parameters() {
        for query in [
            "period=0",
            "period=no",
            "digits=11",
            "algorithm=SHA42",
            "secret=AA",
        ] {
            assert!(
                OTPElement::from_otp_uri(&format!("otpauth://totp/test?secret=JBSWY3DP&{query}"))
                    .is_err()
            );
        }
        assert!(OTPElement::from_otp_uri("otpauth://hotp/test?secret=JBSWY3DP").is_err());
        assert!(OTPElement::from_otp_uri("otpauth://weird/test?secret=JBSWY3DP").is_err());
    }
    #[test]
    fn percent_encoded_ampersand_stays_in_label() {
        let account =
            OTPElement::from_otp_uri("otpauth://totp/Example:user%26name?secret=JBSWY3DP").unwrap();
        assert_eq!(account.label, "user&name");
    }
}
