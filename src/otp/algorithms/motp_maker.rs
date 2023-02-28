use std::time::SystemTime;

use crate::otp::otp_error::OtpError;
use md5::{Digest, Md5};

pub fn motp(secret: &str, pin: &str, period: u64, digits: usize) -> Result<String, OtpError> {
    let seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    get_motp_code(secret, pin, period, digits, seconds)
}

fn get_motp_code(
    secret: &str,
    pin: &str,
    period: u64,
    digits: usize,
    seconds: u64,
) -> Result<String, OtpError> {
    // TODO MOTP Secrets are hex encoded, so do not use BASE32 at all
    let hex_secret = secret;
    let counter = seconds / period;
    let data = format!("{counter}{hex_secret}{pin}");

    let mut md5_hasher = Md5::new();
    md5_hasher.update(data.as_bytes());
    let code = hex::encode(md5_hasher.finalize());
    Ok(code.as_str()[0..digits].to_owned())
}

#[cfg(test)]
mod tests {

    use super::get_motp_code;

    #[test]
    fn test_motp() {
        let seconds: u64 = 165892298;

        assert_eq!(
            Ok("e7d8b6".to_string()),
            get_motp_code("e3152afee62599c8", "1234", 10, 6, seconds)
        )
    }
}
