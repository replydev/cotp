use std::time::SystemTime;

use crate::otp::otp_algorithm::OTPAlgorithm;
use crate::otp::otp_error::OtpError;

use super::hotp_maker::hotp;

pub fn totp(secret: &str, algorithm: OTPAlgorithm, period: u64) -> Result<u32, OtpError> {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| OtpError::InvalidTime)?
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
    let adjusted = time.checked_add_signed(skew).ok_or(OtpError::InvalidTime)?;
    hotp(secret, algorithm, adjusted / time_step)
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

#[cfg(test)]
mod rfc_vectors {
    use super::*;
    use crate::otp::otp_element::format_code;
    use data_encoding::BASE32_NOPAD;

    // RFC 6238, Appendix B. Each hash uses its specified secret length.
    #[test]
    fn all_rfc6238_vectors() {
        let times = [
            59,
            1111111109,
            1111111111,
            1234567890,
            2000000000,
            20000000000,
        ];
        let cases = [
            (
                OTPAlgorithm::Sha1,
                20,
                [
                    "94287082", "07081804", "14050471", "89005924", "69279037", "65353130",
                ],
            ),
            (
                OTPAlgorithm::Sha256,
                32,
                [
                    "46119246", "68084774", "67062674", "91819424", "90698825", "77737706",
                ],
            ),
            (
                OTPAlgorithm::Sha512,
                64,
                [
                    "90693936", "25091201", "99943326", "93441116", "38618901", "47863826",
                ],
            ),
        ];
        for (algorithm, len, expected) in cases {
            let bytes: Vec<u8> = b"1234567890".iter().copied().cycle().take(len).collect();
            let secret = BASE32_NOPAD.encode(&bytes);
            for (time, expected) in times.into_iter().zip(expected) {
                assert_eq!(
                    format_code(8, generate_totp(&secret, algorithm, time, 30, 0).unwrap())
                        .unwrap(),
                    expected
                );
            }
        }
    }

    #[test]
    fn invalid_time_parameters_are_errors() {
        assert_eq!(
            generate_totp("AA", OTPAlgorithm::Sha1, 1, 0, 0),
            Err(OtpError::InvalidPeriod)
        );
        assert_eq!(
            generate_totp("AA", OTPAlgorithm::Sha1, 0, 30, -1),
            Err(OtpError::InvalidTime)
        );
        assert_eq!(
            generate_totp("AA", OTPAlgorithm::Sha1, u64::MAX, 30, 1),
            Err(OtpError::InvalidTime)
        );
    }
}
