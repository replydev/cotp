use std::convert::TryInto;
use std::time::SystemTime;

use data_encoding::BASE32_NOPAD;
use hmac::digest::block_buffer::Eager;
use hmac::digest::core_api::{
    BlockSizeUser, BufferKindUser, CoreProxy, FixedOutputCore, UpdateCore,
};
use hmac::digest::generic_array::typenum::{IsLess, Le, NonZero, U256};
use hmac::digest::HashMarker;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

pub fn totp(secret: &str, algorithm: &str) -> Result<u32, String> {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    generate_totp(secret, algorithm, time, 30, 0)
}

fn generate_totp(
    secret: &str,
    algorithm: &str,
    time: u64,
    time_step: u64,
    skew: i64,
) -> Result<u32, String> {
    hotp(secret, algorithm, ((time as i64 + skew) as u64) / time_step)
}

pub fn hotp(secret: &str, algorithm: &str, counter: u64) -> Result<u32, String> {
    match algorithm {
        "SHA256" => generate_hotp::<Sha256>(secret, counter),
        "SHA512" => generate_hotp::<Sha512>(secret, counter),
        _ => generate_hotp::<Sha1>(secret, counter),
    }
}

fn generate_hotp<D>(secret: &str, counter: u64) -> Result<u32, String>
where
    D: CoreProxy,
    D::Core: HashMarker
        + UpdateCore
        + FixedOutputCore
        + BufferKindUser<BufferKind = Eager>
        + Default
        + Clone,
    <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
    Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero,
{
    // decode the base32 secret
    let secret_decoded = match BASE32_NOPAD.decode(secret.as_bytes()) {
        Ok(result) => result,
        Err(e) => return Err(format!("{:?}", e)),
    };
    // calculate HMAC from secret bytes and counter
    let mut hmac: Hmac<D> =
        Hmac::new_from_slice(secret_decoded.as_slice()).expect("Failed to derive HMAC");
    hmac.update(&counter.to_be_bytes());
    let hash = hmac.finalize().into_bytes();

    // calculate offset
    let offset: usize = match hash.last() {
        Some(result) => *result & 0xf,
        None => return Err(String::from("Invalid digest")),
    } as usize;

    // calculate code
    let code_bytes: [u8; 4] = match hash[offset..offset + 4].try_into() {
        Ok(x) => x,
        Err(_) => return Err(String::from("Invalid digest")),
    };
    Ok(u32::from_be_bytes(code_bytes) & 0x7fffffff)
}

#[cfg(test)]
mod tests {
    use crate::otp::otp_helper::format_code;
    use crate::otp::otp_maker::{generate_hotp, generate_totp};
    use sha1::Sha1;

    #[test]
    fn test_totp() {
        assert_eq!(
            format_code(
                generate_totp("BASE32SECRET3232", "SHA1", 0, 30, 0).unwrap(),
                6
            ),
            "260182"
        );
    }

    #[test]
    fn test_hotp() {
        assert_eq!(
            format_code(generate_hotp::<Sha1>("BASE32SECRET3232", 0).unwrap(), 6),
            "260182"
        );
        assert_eq!(
            format_code(generate_hotp::<Sha1>("BASE32SECRET3232", 1).unwrap(), 6),
            "055283"
        );
    }
}
