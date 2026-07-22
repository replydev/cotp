use std::time::SystemTime;

use crate::otp::otp_algorithm::OTPAlgorithm;
use crate::otp::otp_error::OtpError;

use super::hotp_maker::hotp;

pub fn totp(secret: &str, algorithm: OTPAlgorithm, period: u64) -> Result<u32, OtpError> {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    generate_totp(secret, algorithm, time, period, 0)
}

fn generate_totp(
    secret: &str,
    algorithm: OTPAlgorithm,
    time: u64,
    time_step: u64,
    skew: i64,
) -> Result<u32, OtpError> {
    if time_step == 0 {
        return Err(OtpError::InvalidPeriod);
    }
    hotp(secret, algorithm, ((time as i64 + skew) as u64) / time_step)
}

#[cfg(test)]
mod tests {
    use crate::otp::{
        algorithms::totp_maker::generate_totp, otp_algorithm::OTPAlgorithm,
        otp_element::format_code,
    };

    #[test]
    fn test_totp() {
        assert_eq!(
            455260182,
            generate_totp("BASE32SECRET3232", OTPAlgorithm::Sha1, 0, 30, 0).unwrap()
        );
    }

    #[test]
    fn test_totp_with_60_seconds_period() {
        // Arrange / Act
        let raw_code = generate_totp(
            "DEADBEEFDEADBEEFDEADBEEFDEADBEEF",
            OTPAlgorithm::Sha1,
            1777799540,
            60,
            0,
        )
        .unwrap();

        let code = format_code(6, raw_code).unwrap();

        // Assert
        assert_eq!("295439", code)
    }
}
