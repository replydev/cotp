use std::time::SystemTime;

use super::{hotp_maker::hotp, otp_element::OTPAlgorithm};

pub fn totp(secret: &str, algorithm: OTPAlgorithm) -> Result<u32, String> {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    generate_totp(secret, algorithm, time, 30, 0)
}

fn generate_totp(
    secret: &str,
    algorithm: OTPAlgorithm,
    time: u64,
    time_step: u64,
    skew: i64,
) -> Result<u32, String> {
    hotp(secret, algorithm, ((time as i64 + skew) as u64) / time_step)
}

#[cfg(test)]
mod tests {

    use crate::otp::{otp_element::OTPAlgorithm, totp_maker::generate_totp};

    #[test]
    fn test_totp() {
        assert_eq!(
            format_code(
                generate_totp("BASE32SECRET3232", OTPAlgorithm::Sha1, 0, 30, 0).unwrap(),
                6
            ),
            "260182"
        );
    }

    fn format_code(value: u32, digits: u32) -> String {
        // Get the formatted code
        let s = (value % 10_u32.pow(digits)).to_string();
        "0".repeat(digits as usize - s.chars().count()) + s.as_str()
    }
}
