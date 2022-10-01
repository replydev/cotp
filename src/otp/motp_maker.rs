use std::time::SystemTime;

use md5::{Digest, Md5};

use super::otp_element::OTPElement;

pub fn motp(element: &OTPElement) -> Result<String, String> {
    let seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    get_motp_code(element, seconds)
}

fn get_motp_code(element: &OTPElement, seconds: u64) -> Result<String, String> {
    match element.pin() {
        Some(v) => {
            // TODO MOTP Secrets are hex encoded, so do not use BASE32 at all
            let hex_secret = element.secret();
            let counter = seconds / element.period();
            let data = format!("{}{}{}", counter, hex_secret, v);

            let mut md5_hasher = Md5::new();
            md5_hasher.update(data.as_bytes());
            let code = hex::encode(md5_hasher.finalize());
            Ok(code.as_str()[0..element.digits() as usize].to_owned())
        }
        None => Err(String::from("MOTP codes require a pin value")),
    }
}

#[cfg(test)]
mod tests {
    use crate::otp::otp_element::OTPElement;

    use super::get_motp_code;

    #[test]
    fn test_motp() {
        let otp_element = OTPElement::new(
            "e3152afee62599c8".to_string(),
            "".to_string(),
            "label".to_string(),
            6,
            "MOTP".to_string(),
            "MD5".to_string(),
            10,
            None,
            Some("1234".to_string()),
        );
        let seconds: u64 = 165892298;

        assert_eq!(
            Ok("e7d8b6".to_string()),
            get_motp_code(&otp_element, seconds)
        )
    }
}
