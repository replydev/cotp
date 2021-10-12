use std::fs::read_to_string;

use serde::Deserialize;
use serde_json;

use crate::otp::otp_element::OTPElement;

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
    period: u64,
}

pub fn import(filepath: &str) -> Result<Vec<OTPElement>, String> {
    let file_to_import_contents = match read_to_string(filepath){
        Ok(result) => result,
        Err(e) => return Err(format!("Error during file reading: {:?}",e)),
    };
    let result: Result<AegisJson, serde_json::Error> = serde_json::from_str(&file_to_import_contents);
    let aegis;
    match result {
        Ok(element) => aegis = element,
        Err(e) => return Err(format!("{}", e)),
    }

    let mut elements: Vec<OTPElement> = Vec::new();

    for i in 0..aegis.db.entries.len() {
        elements.push(OTPElement::new(
            String::from(&aegis.db.entries[i].info.secret),
            String::from(&aegis.db.entries[i].issuer),
            String::from(&aegis.db.entries[i].name),
            aegis.db.entries[i].info.digits,
            String::from(&aegis.db.entries[i]._type),
            String::from(&aegis.db.entries[i].info.algo),
            String::from(""),
            0,
            0,
            aegis.db.entries[i].info.period,
            vec![]))
    }
    Ok(elements)
}