use std::fs::read_to_string;

use serde::Deserialize;
use serde_json;

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

#[derive(Deserialize)]
struct AegisJson {
    //version: u64,
    //header: AegisHeader,
    db: AegisDb,
}

#[derive(Deserialize)]
struct AegisHeader {
    //slots: Option<String>,
    //params: Option<String>,
}

#[derive(Deserialize)]
struct AegisDb {
    //version: u64,
    entries: Vec<AegisElement>,
}

#[derive(Deserialize)]
struct AegisElement {
    #[serde(rename = "type")]
    _type: String,
    //uuid: String,
    name: String,
    issuer: String,
    //icon: Option<String>,
    info: AegisInfo,
}

#[derive(Deserialize)]
struct AegisInfo {
    secret: String,
    algo: String,
    digits: u64,
    period: Option<u64>,
    counter: Option<u64>,
}

pub fn import(filepath: &str) -> Result<Vec<OTPElement>, String> {
    let file_to_import_contents = match read_to_string(filepath) {
        Ok(result) => result,
        Err(e) => return Err(format!("Error during file reading: {:?}", e)),
    };
    import_from_string(file_to_import_contents.as_str())
}

pub fn import_from_string(file_to_import_contents: &str) -> Result<Vec<OTPElement>, String> {
    match serde_json::from_str::<AegisJson>(file_to_import_contents) {
        Ok(element) => Ok(map_entries(element.db.entries)),
        Err(_) => {
            let aegis_db: AegisDb = match serde_json::from_str(file_to_import_contents) {
                Ok(element) => element,
                Err(e) => return Err(format!("{:?}", e)),
            };
            // maybe we are importing from an encrypted aegis database, so we don
            Ok(map_entries(aegis_db.entries))
        }
    }
}

fn map_entries(entries: Vec<AegisElement>) -> Vec<OTPElement> {
    entries
        .into_iter()
        .map(|e| OTPElement {
            secret: e.info.secret,
            issuer: e.issuer,
            label: e.name,
            digits: e.info.digits,
            type_: OTPType::from(e._type.as_str()),
            algorithm: OTPAlgorithm::from(e.info.algo.as_str()),
            period: e.info.period.unwrap_or(30),
            counter: e.info.counter,
            pin: None,
        })
        .collect()
}
