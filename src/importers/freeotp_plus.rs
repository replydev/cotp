use std::fs::read_to_string;

use data_encoding::BASE32_NOPAD;
use serde::Deserialize;
use serde_json;

use crate::otp::otp_element::OTPElement;

#[derive(Deserialize)]
struct FreeOTPPlusJson{
    #[serde(rename = "tokenOrder")]
    token_order: Vec<String>,
    tokens: Vec<FreeOTPElement>,
}

#[derive(Deserialize)]
struct FreeOTPElement {
    algo: String,
    counter: u64,
    digits: u64,
    #[serde(rename = "issuerExt")]
    issuer_ext: String,
    #[serde(rename = "label")]
    _label: String,
    period: u64,
    secret: Vec<i8>,
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
    for label in freeotp.token_order {
        elements.push(
            OTPElement::new(
                encode_secret(&freeotp.tokens[i].secret),
                freeotp.tokens[i].issuer_ext.clone(),
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

fn encode_secret(secret: &Vec<i8>) -> String {
    BASE32_NOPAD.encode(secret
        .iter()
        .map(|n| *n as u8)
        .collect::<Vec<u8>>()
        .as_slice()
    )
}

#[cfg(test)]
mod tests{
    use super::encode_secret;

    #[test]
    fn test_secret_conversion(){
        let secret: Vec<i8> =
        vec![-34,104,-37,-33,82,-89,-93,-105,-14,5,100,-73,-84,1,11,73,101,-92
        ,106,122,-90,111,-119,30,87,-6,16,-57,-126,25,0,-65,-35,-76,-38];

        assert_eq!(encode_secret(&secret), String::from("3ZUNXX2SU6RZP4QFMS32YAILJFS2I2T2UZXYSHSX7IIMPAQZAC753NG2"));
    }
}

