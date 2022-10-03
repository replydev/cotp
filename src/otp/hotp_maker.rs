use std::convert::TryInto;
use std::time::SystemTime;

use aes_gcm::aead::generic_array::GenericArray;
use data_encoding::BASE32_NOPAD;
use hmac::digest::block_buffer::Eager;
use hmac::digest::core_api::{
    BlockSizeUser, BufferKindUser, CoreProxy, FixedOutputCore, UpdateCore,
};
use hmac::digest::generic_array::typenum::{IsLess, Le, NonZero, U256};
use hmac::digest::{HashMarker, OutputSizeUser};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use super::otp_element::OTPAlgorithm;

pub fn hotp(secret: &str, algorithm: OTPAlgorithm, counter: u64) -> Result<u32, String> {
    match algorithm.to_string().to_uppercase().as_str() {
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

    let hash = hotp_hash::<D>(&secret_decoded, counter);

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

pub fn hotp_hash<D>(
    secret: &[u8],
    counter: u64,
) -> GenericArray<u8, <<D as CoreProxy>::Core as OutputSizeUser>::OutputSize>
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
    // calculate HMAC from secret bytes and counter
    let mut hmac: Hmac<D> = Hmac::new_from_slice(secret).expect("Failed to derive HMAC");
    hmac.update(&counter.to_be_bytes());
    hmac.finalize().into_bytes()
}

#[cfg(test)]
mod tests {
    use sha1::Sha1;

    use crate::otp::hotp_maker::generate_hotp;

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

    fn format_code(value: u32, digits: u32) -> String {
        // Get the formatted code
        let s = (value % 10_u32.pow(digits)).to_string();
        "0".repeat(digits as usize - s.chars().count()) + s.as_str()
    }
}
