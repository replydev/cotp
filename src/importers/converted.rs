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

impl TryFrom<ConvertedJson> for OTPElement {
    type Error = String;

    fn try_from(converted_json: ConvertedJson) -> Result<Self, Self::Error> {
        let type_ = OTPType::try_from(converted_json.type_.as_str()).map_err(|e| e.to_string())?;
        let algorithm =
            OTPAlgorithm::try_from(converted_json.algorithm.as_str()).map_err(|e| e.to_string())?;
        let counter: Option<u64> = (type_ == OTPType::Hotp).then_some(converted_json.counter);
        Ok(OTPElement {
            secret: converted_json.secret,
            issuer: converted_json.issuer.unwrap_or_default(),
            label: converted_json.label.unwrap_or_default(),
            digits: converted_json.digits,
            type_,
            algorithm,
            period: 30,
            counter,
            pin: None,
        })
    }
}

// Newtype pattern to permit impl to Vec<OTPElement>
#[derive(Deserialize)]
pub struct ConvertedJsonList(Vec<ConvertedJson>);

impl TryFrom<ConvertedJsonList> for Vec<OTPElement> {
    type Error = String;
    fn try_from(value: ConvertedJsonList) -> Result<Self, Self::Error> {
        value.0.into_iter().map(TryInto::try_into).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ConvertedJsonList;
    use crate::otp::otp_element::OTPElement;

    #[test]
    fn unknown_type_in_import_file_is_a_clear_error() {
        let json = r#"[
            {
                "label": "Label",
                "secret": "AAAAAAAAAAAAAAAA",
                "issuer": "Issuer",
                "type": "OCRA",
                "algorithm": "SHA1",
                "digits": 6,
                "counter": 0
            }
        ]"#;

        let deserialized: ConvertedJsonList = serde_json::from_str(json).unwrap();
        let result: Result<Vec<OTPElement>, String> = deserialized.try_into();

        let error = result.unwrap_err();
        assert!(error.contains("Unknown OTP type"));
        assert!(error.contains("OCRA"));
    }
}
