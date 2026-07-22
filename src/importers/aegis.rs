use serde::Deserialize;

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

#[derive(Deserialize)]
pub struct AegisJson {
    //version: u64,
    //header: AegisHeader,
    db: AegisDb,
}

#[derive(Deserialize)]
pub(crate) struct AegisDb {
    //version: u64,
    entries: Vec<AegisElement>,
}

#[derive(Deserialize)]
struct AegisElement {
    r#type: String,
    //uuid: String,
    name: String,
    issuer: String,
    //icon: Option<String>,
    info: AegisInfo,
}

impl TryFrom<AegisElement> for OTPElement {
    type Error = String;

    fn try_from(value: AegisElement) -> Result<Self, Self::Error> {
        let type_ = OTPType::try_from(value.r#type.as_str()).map_err(|e| e.to_string())?;
        let algorithm =
            OTPAlgorithm::try_from(value.info.algo.as_str()).map_err(|e| e.to_string())?;
        Ok(OTPElement {
            secret: value.info.secret,
            issuer: value.issuer,
            label: value.name,
            digits: value.info.digits,
            type_,
            algorithm,
            period: value.info.period.unwrap_or(30),
            counter: value.info.counter,
            pin: value.info.pin,
        })
    }
}

impl TryFrom<AegisDb> for Vec<OTPElement> {
    type Error = String;

    fn try_from(aegis_db: AegisDb) -> Result<Self, Self::Error> {
        aegis_db
            .entries
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }
}

impl TryFrom<AegisJson> for Vec<OTPElement> {
    type Error = String;

    fn try_from(aegis_json: AegisJson) -> Result<Self, Self::Error> {
        aegis_json.db.try_into()
    }
}

#[derive(Deserialize)]
struct AegisInfo {
    secret: String,
    algo: String,
    digits: u64,
    period: Option<u64>,
    counter: Option<u64>,
    #[serde(default)]
    pin: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::AegisJson;
    use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

    #[test]
    fn test_pin_is_imported() {
        let json = r#"{
            "version": 1,
            "header": {"slots": null, "params": null},
            "db": {
                "version": 2,
                "entries": [
                    {
                        "type": "yandex",
                        "uuid": "00000000-0000-0000-0000-000000000000",
                        "name": "Label",
                        "issuer": "Issuer",
                        "info": {
                            "secret": "AAAAAAAAAAAAAAAA",
                            "algo": "SHA256",
                            "digits": 8,
                            "period": 30,
                            "pin": "1234"
                        }
                    },
                    {
                        "type": "totp",
                        "uuid": "00000000-0000-0000-0000-000000000001",
                        "name": "Label2",
                        "issuer": "Issuer2",
                        "info": {
                            "secret": "BBBBBBBBBBBBBBBB",
                            "algo": "SHA1",
                            "digits": 6,
                            "period": 30
                        }
                    }
                ]
            }
        }"#;

        let deserialized: AegisJson = serde_json::from_str(json).unwrap();
        let elements: Vec<OTPElement> = deserialized.try_into().unwrap();

        assert_eq!(
            vec![
                OTPElement {
                    secret: "AAAAAAAAAAAAAAAA".to_string(),
                    issuer: "Issuer".to_string(),
                    label: "Label".to_string(),
                    digits: 8,
                    type_: OTPType::Yandex,
                    algorithm: OTPAlgorithm::Sha256,
                    period: 30,
                    counter: None,
                    pin: Some("1234".to_string()),
                },
                OTPElement {
                    secret: "BBBBBBBBBBBBBBBB".to_string(),
                    issuer: "Issuer2".to_string(),
                    label: "Label2".to_string(),
                    digits: 6,
                    type_: OTPType::Totp,
                    algorithm: OTPAlgorithm::Sha1,
                    period: 30,
                    counter: None,
                    pin: None,
                }
            ],
            elements
        );
    }
}
