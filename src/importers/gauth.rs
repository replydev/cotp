use crate::otp::otp_element::OTPElement;
use std::fs::read_to_string;
use serde_json;
use serde::Deserialize;

#[derive(Deserialize)]
struct GAuthJson{
    label: Option<String>,
    secret: String,
    issuer: Option<String>,
}

pub fn import(filepath: &str) -> Result<Vec<OTPElement>,String> {
    let file_to_import_contents = read_to_string(filepath).unwrap();
    let result: Result<Vec<GAuthJson>,serde_json::Error> = serde_json::from_str(&file_to_import_contents);
    let gauth: Vec<GAuthJson>;

    match result{
        Ok(r) => gauth = r,
        Err(e) => return Err(format!("{}",e)),
    }

    let mut elements: Vec<OTPElement> = Vec::new();

    for i in 0..gauth.len(){
        let secret = gauth[i].secret.to_owned();
        let issuer = gauth[i].issuer.to_owned().unwrap_or_default();
        let label = gauth[i].label.to_owned().unwrap_or_default();
        elements.push(OTPElement::new(
            secret,
            issuer,
            label,
            6,
            String::from("TOTP"), 
            String::from("SHA1"), 
            String::from(""), 
            0, 
            0, 
            30, 
            vec![]))
    }
    Ok(elements)
}
