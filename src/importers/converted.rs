use serde::Deserialize;

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

#[derive(Deserialize)]
struct ConvertedJson {
    label: Option<String>,
    secret: String,
    issuer: Option<String>,
    #[serde(rename = "type")]
    type_: String,
    algorithm: String,
    digits: u64,
    counter: u64,
}

impl From<ConvertedJson> for OTPElement {
    fn from(converted_json: ConvertedJson) -> Self {
        let counter: Option<u64> = (OTPType::from(converted_json.type_.as_str()) == OTPType::Hotp)
            .then_some(converted_json.counter);
        OTPElement {
            secret: converted_json.secret,
            issuer: converted_json.issuer.unwrap_or_default(),
            label: converted_json.label.unwrap_or_default(),
            digits: converted_json.digits,
            type_: OTPType::from(converted_json.type_.as_str()),
            algorithm: OTPAlgorithm::from(converted_json.algorithm.as_str()),
            period: 30,
            counter,
            pin: None,
        }
    }
}

// Newtype pattern to permit impl to Vec<OTPElement>
#[derive(Deserialize)]
pub struct ConvertedJsonList(Vec<ConvertedJson>);

impl TryFrom<ConvertedJsonList> for Vec<OTPElement> {
    type Error = String;
    fn try_from(value: ConvertedJsonList) -> Result<Self, Self::Error> {
        Ok(value.0.into_iter().map(|e| e.into()).collect())
    }
}
