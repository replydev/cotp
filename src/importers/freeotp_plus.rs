use std::fs::read_to_string;

use data_encoding::BASE32_NOPAD;
use serde::Deserialize;
use serde_json;

use crate::otp::otp_element::OTPElement;

#[derive(Deserialize)]
struct FreeOTPPlusJson{
    tokenOrder: Vec<String>,
    tokens: Vec<FreeOTPElement>,
}

#[derive(Deserialize)]
struct FreeOTPElement {
    algo: String,
    counter: u64,
    digits: u64,
    issuerExt: String,
    label: String,
    period: u64,
    secret: Vec<i16>,
    #[serde(rename = "type")]
    _type: String,
}


pub fn import(file_path: &str) -> Result<Vec<OTPElement>,String>{
    let json = match read_to_string(file_path) {
        Ok(r) => r,
        Err(e) => return Err(format!("{:?}",e)),
    };

    let freeotp: FreeOTPPlusJson = match serde_json::from_str(json.as_str()){
        Ok(r) => r,
        Err(e)=> return Err(format!("Error during deserializing: {:?}",e)),
    };

    let mut i = 0;
    let mut elements = Vec::<OTPElement>::new();
    for label in freeotp.tokenOrder{
        elements.push(
            OTPElement::new(
                i16_secret_to_base32(&freeotp.tokens[i].secret),
                freeotp.tokens[i].issuerExt.clone(),
                label,
                freeotp.tokens[i].digits,
                freeotp.tokens[i]._type.clone(),
                freeotp.tokens[i].algo.clone(),
                String::from(""),
                0,
                0,
                freeotp.tokens[i].period,
                freeotp.tokens[i].counter,
                vec![])
        );
        i += 1;
    }

    Ok(elements)
}


fn i16_secret_to_base32(secret: &Vec<i16>) -> String {
    let mut converted_vec: Vec<u8> = Vec::with_capacity(secret.len());
    for i in secret {
        // convert to u8 by adding 127
        converted_vec.push(*i.to_ne_bytes().first().unwrap_or(&0u8));
    }
    BASE32_NOPAD.encode(&converted_vec.as_slice())
}
