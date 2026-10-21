use serde::{Deserialize, Serialize};

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

#[derive(Serialize, Deserialize)]
pub struct AegisJson {
    //version: u64,
    //header: AegisHeader,
    db: AegisDb,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AegisHeader {
    //slots: Option<String>,
    //params: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AegisDb {
    //version: u64,
    entries: Vec<AegisElement>,
    #[serde(default)]
    groups: Vec<AegisGroup>,
}

#[derive(Serialize, Deserialize)]
struct AegisGroup {
    uuid: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct AegisElement {
    r#type: String,
    //uuid: String,
    name: String,
    issuer: String,
    //icon: Option<String>,
    info: AegisInfo,
    #[serde(default)]
    favorite: bool,
    #[serde(default)]
    groups: Vec<String>,
}

impl From<AegisElement> for OTPElement {
    fn from(value: AegisElement) -> Self {
        OTPElement {
            id: String::new(),
            alias: None,
            tags: Vec::new(),
            favorite: value.favorite,
            order: None,
            secret: value.info.secret,
            issuer: value.issuer,
            label: value.name,
            digits: value.info.digits,
            type_: OTPType::from(value.r#type.as_str()),
            algorithm: OTPAlgorithm::from(value.info.algo.as_str()),
            period: value.info.period.unwrap_or(30),
            counter: value.info.counter,
            pin: None,
        }
    }
}

impl TryFrom<AegisDb> for Vec<OTPElement> {
    type Error = String;

    fn try_from(aegis_db: AegisDb) -> Result<Self, Self::Error> {
        Ok(aegis_db
            .entries
            .into_iter()
            .map(|entry| {
                let tags = entry
                    .groups
                    .iter()
                    .filter_map(|id| {
                        aegis_db
                            .groups
                            .iter()
                            .find(|g| &g.uuid == id)
                            .map(|g| g.name.clone())
                    })
                    .collect();
                let mut element: OTPElement = entry.into();
                element.tags = tags;
                element
            })
            .collect())
    }
}

impl TryFrom<AegisJson> for Vec<OTPElement> {
    type Error = String;

    fn try_from(aegis_json: AegisJson) -> Result<Self, Self::Error> {
        aegis_json.db.try_into()
    }
}

#[derive(Serialize, Deserialize)]
struct AegisInfo {
    secret: String,
    algo: String,
    digits: u64,
    period: Option<u64>,
    counter: Option<u64>,
}
