use oath::{totp_raw_now, HashType};
use data_encoding::BASE32_NOPAD;

pub fn make_totp<'a>(secret: &str, algorithm: &str, digits: u64) -> u64 {
    let secret_bytes = BASE32_NOPAD.decode(secret.as_bytes()).expect("Failed to decode BASE32 Secret");
    return match algorithm {
        "SHA256" => totp_raw_now(&*secret_bytes, digits as u32, 0, 30, &HashType::SHA256),
        "SHA512" => totp_raw_now(&*secret_bytes, digits as u32, 0, 30, &HashType::SHA512),
        _ => totp_raw_now(&*secret_bytes, digits as u32, 0, 30, &HashType::SHA1),
    }
}
