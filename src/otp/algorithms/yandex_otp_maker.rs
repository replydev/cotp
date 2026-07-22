// Ported from https://github.com/beemdevelopment/Aegis/blob/3d13117752491de81d3779dae34407c651954f7b/app/src/main/java/com/beemdevelopment/aegis/crypto/otp/YAOTP.java

use std::time::SystemTime;

use data_encoding::BASE32_NOPAD;
use sha2::{Digest, Sha256};

use crate::otp::otp_error::OtpError;

use super::hotp_maker::hotp_hash;

const EN_ALPHABET_LENGTH: u64 = 26;
const SECRET_LENGTH: usize = 16;

/// Yandex OTP codes are always calculated with HMAC-SHA256, regardless of the
/// algorithm stored on the element. The Yandex.Key app (and Aegis' reference
/// implementation this port is based on) exclusively use HMAC-SHA256.
///
/// Honoring the element's algorithm field was also unsound: HMAC-SHA1 output
/// is only 20 bytes, while the dynamic offset below can reach 15, making
/// `hash[offset..offset + 8]` read out of bounds for offsets 13..=15.
pub fn yandex(secret: &str, pin: &str, period: u64, digits: usize) -> Result<String, OtpError> {
    let seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    calculate_yandex_code(secret, pin, period, digits, seconds)
}

fn calculate_yandex_code(
    secret: &str,
    pin: &str,
    period: u64,
    digits: usize,
    seconds: u64,
) -> Result<String, OtpError> {
    if period == 0 {
        return Err(OtpError::InvalidPeriod);
    }

    let decoded_secret = match BASE32_NOPAD.decode(secret.as_bytes()) {
        Ok(r) => r,
        Err(e) => return Err(OtpError::SecretEncoding(e.kind, e.position)),
    };

    if decoded_secret.len() < SECRET_LENGTH {
        return Err(OtpError::ShortSecret);
    }

    let parsed_secret = &decoded_secret.as_slice()[0..SECRET_LENGTH];

    let mut pin_with_secret: Vec<u8> = Vec::with_capacity(pin.len() + SECRET_LENGTH);

    pin_with_secret.append(&mut pin.as_bytes().to_vec());
    pin_with_secret.append(&mut parsed_secret.to_vec());

    let mut sha256 = Sha256::new();
    sha256.update(pin_with_secret);
    let mut key_hash = &sha256.finalize()[..];
    if key_hash[0] == 0u8 {
        key_hash = &key_hash[1..];
    }

    let counter: u64 = seconds / period;
    let mut period_hash = hotp_hash::<Sha256>(key_hash, counter);

    // calculate offset
    let offset: usize = match period_hash.last() {
        Some(result) => *result & 0xf,
        None => return Err(OtpError::InvalidOffset),
    } as usize;

    *period_hash.get_mut(offset).ok_or(OtpError::InvalidOffset)? &= 0x7f;

    // calculate code, bounds-checked as defense in depth
    let code_bytes: [u8; 8] = period_hash
        .get(offset..offset + 8)
        .ok_or(OtpError::InvalidDigest)?
        .try_into()
        .map_err(|_| OtpError::InvalidDigest)?;

    let code = u64::from_be_bytes(code_bytes);

    Ok(to_yandex_string(code, digits))
}

fn to_yandex_string(mut code: u64, digits: usize) -> String {
    code %= EN_ALPHABET_LENGTH.pow(digits as u32);
    let mut s = String::with_capacity(digits);
    let mut i: isize = digits as isize - 1;
    while i >= 0 {
        let c = char::from_u32(u32::from('a') + ((code % EN_ALPHABET_LENGTH) as u32));
        s.push(c.unwrap_or('❤'));
        code /= EN_ALPHABET_LENGTH;
        i -= 1;
    }

    s.chars().rev().collect::<String>().to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::calculate_yandex_code;

    #[test]
    fn test_yandex() {
        let seconds: u64 = 1641559648;

        assert_eq!(
            calculate_yandex_code(
                "6SB2IKNM6OBZPAVBVTOHDKS4FAAAAAAADFUTQMBTRY",
                "5239",
                30,
                8,
                seconds
            )
            .unwrap(),
            "umozdicq".to_uppercase()
        );
    }
}
