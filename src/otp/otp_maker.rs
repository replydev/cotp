// inspired and modified from original: from: https://github.com/TimDumol/rust-otp

use std::time::{SystemTime, SystemTimeError};
use std::convert::TryInto;
use data_encoding::{BASE32_NOPAD, DecodeError};
use err_derive::Error;
use ring::hmac;
use crate::utils;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display="invalid time provided")]
    InvalidTimeError(#[error(source)] SystemTimeError),
    #[error(display="invalid digest provided: {:?}", _0)]
    InvalidDigest(Vec<u8>),
    #[error(display="invalid secret provided")]
    InvalidSecret(#[error(source)] DecodeError)
}

/// Decodes a secret (given as an RFC4648 base32-encoded ASCII string)
/// into a byte string
fn decode_secret(secret: &str) -> Result<Vec<u8>, DecodeError> {
    BASE32_NOPAD.decode(secret.as_bytes())
}

/// Calculates the HMAC digest for the given secret and counter.
fn calc_digest(decoded_secret: &[u8], counter: u64,algorithm: &str) -> hmac::Tag {
    let key = match algorithm{
        "SHA256" => hmac::Key::new(hmac::HMAC_SHA256, decoded_secret),
        "SHA512" => hmac::Key::new(hmac::HMAC_SHA512, decoded_secret),
        _=> hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, decoded_secret),
    };
    hmac::sign(&key, &counter.to_be_bytes())
}

/// Encodes the HMAC digest into a 6-digit integer.
fn encode_digest(digest: &[u8],digits: u64) -> Result<u32, Error> {
    let offset = match digest.last() {
        Some(x) => *x & 0xf,
        None => return Err(Error::InvalidDigest(Vec::from(digest)))
    } as usize;
    let code_bytes: [u8; 4] = match digest[offset..offset+4].try_into() {
        Ok(x) => x,
        Err(_) => return Err(Error::InvalidDigest(Vec::from(digest)))
    };
    let code = u32::from_be_bytes(code_bytes);
    Ok((code & 0x7fffffff) % (utils::pow(10.0, digits as i64) as u32))
}

/// Performs the [HMAC-based One-time Password Algorithm](http://en.wikipedia.org/wiki/HMAC-based_One-time_Password_Algorithm)
/// (HOTP) given an RFC4648 base32 encoded secret, and an integer counter.
pub fn make_hotp(secret: &str, counter: u64,algorithm: &str,digits: u64) -> Result<u32, Error> {
    let decoded = decode_secret(secret)?;
    encode_digest(calc_digest(decoded.as_slice(), counter,algorithm).as_ref(),digits)
}

/// Helper function for `make_totp` to make it testable. Note that times
/// before Unix epoch are not supported.
fn make_totp_helper(secret: &str, time_step: u64, skew: i64, time: u64,algorithm: &str,digits: u64) -> Result<u32, Error> {
    let counter = ((time as i64 + skew) as u64) / time_step;
    make_hotp(secret, counter,algorithm,digits)
}

/// Performs the [Time-based One-time Password Algorithm](http://en.wikipedia.org/wiki/Time-based_One-time_Password_Algorithm)
/// (TOTP) given an RFC4648 base32 encoded secret, the time step in seconds,
/// and a skew in seconds.
pub fn make_totp(secret: &str, time_step: u64, skew: i64,algorithm: &str,digits: u64) -> Result<u32, Error> {
    let now = SystemTime::now();
    let time_since_epoch = now.duration_since(SystemTime::UNIX_EPOCH)?;
    match make_totp_helper(secret, time_step, skew, time_since_epoch.as_secs(), algorithm,digits) {
        Ok(d) => Ok(d),
        Err(err) => return Err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::{make_hotp, make_totp_helper};
    #[test]
    fn hotp() {
        assert_eq!(make_hotp("BASE32SECRET3232", 0,"SHA1",6).unwrap(), 260182);
        assert_eq!(make_hotp("BASE32SECRET3232", 1,"SHA1",6).unwrap(), 55283);
        assert_eq!(make_hotp("BASE32SECRET3232", 1401,"SHA1",6).unwrap(), 316439);
    }

    #[test]
    fn totp() {
        assert_eq!(make_totp_helper("BASE32SECRET3232", 30, 0, 0,"SHA1",6).unwrap(), 260182);
        assert_eq!(make_totp_helper("BASE32SECRET3232", 3600, 0, 7,"SHA1",6).unwrap(), 260182);
        assert_eq!(make_totp_helper("BASE32SECRET3232", 30, 0, 35,"SHA1",6).unwrap(), 55283);
        assert_eq!(make_totp_helper("BASE32SECRET3232", 1, -2, 1403,"SHA1",6).unwrap(), 316439);
    }
}