// Ported from https://github.com/beemdevelopment/Aegis/blob/master/app/src/main/java/com/beemdevelopment/aegis/crypto/otp/OTP.java

use crate::otp::otp_algorithm::OTPAlgorithm;
use crate::otp::otp_error::OtpError;

use super::totp_maker::totp;

const STEAM_ALPHABET: &str = "23456789BCDFGHJKMNPQRTVWXY";

pub fn steam(secret: &str, algorithm: OTPAlgorithm, digits: usize) -> Result<String, OtpError> {
    totp(secret, algorithm).map(|v| to_steam_string(v as usize, digits))
}

fn to_steam_string(mut code: usize, digits: usize) -> String {
    let mut res: String = String::with_capacity(digits);
    let alphabet_len = STEAM_ALPHABET.chars().count();

    for _ in 0..digits {
        let c = STEAM_ALPHABET.chars().nth(code % alphabet_len).unwrap();
        res.push(c);
        code /= alphabet_len;
    }
    res
}

#[cfg(test)]
mod tests {
    use super::to_steam_string;

    #[test]
    fn test_steam_code() {
        assert_eq!(to_steam_string(36751792, 5), String::from("GJ2F4"))
    }
}
