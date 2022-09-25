use std::time::SystemTime;

use aes_gcm::aes::cipher::BlockSizeUser;
use chacha20poly1305::consts::U256;

use data_encoding::BASE32_NOPAD;
use hmac::digest::{
    block_buffer::Eager,
    core_api::{BufferKindUser, CoreProxy, FixedOutputCore, UpdateCore},
    typenum::{IsLess, Le, NonZero},
    HashMarker,
};
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};

use super::{otp_element::OTPElement, otp_maker::hotp_hash};

const EN_ALPHABET_LENGTH: u64 = 26;
const SECRET_LENGHT: usize = 16;

pub fn yandex(element: &OTPElement) -> Result<String, String> {
    let seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    match element.algorithm().to_uppercase().as_str() {
        "SHA256" => calculate_yandex_code::<Sha256>(element, seconds),
        "SHA512" => calculate_yandex_code::<Sha512>(element, seconds),
        _ => calculate_yandex_code::<Sha1>(element, seconds),
    }
}

fn calculate_yandex_code<D>(element: &OTPElement, seconds: u64) -> Result<String, String>
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
    match element.yandex_pin() {
        Some(pin) => {
            let decoded_secret = match BASE32_NOPAD.decode(element.secret().as_bytes()) {
                Ok(r) => r,
                Err(_) => return Err(String::from("Error during secret parsing")),
            };

            let parsed_secret = &decoded_secret.as_slice()[0..SECRET_LENGHT];

            let mut pin_with_secret: Vec<u8> =
                Vec::with_capacity(pin.as_bytes().len() + SECRET_LENGHT);

            pin_with_secret.append(&mut pin.as_bytes().to_vec());
            pin_with_secret.append(&mut parsed_secret.to_vec());

            let mut sha256 = Sha256::new();
            sha256.update(pin_with_secret);
            let mut key_hash = &sha256.finalize()[..];
            if key_hash[0] == 0u8 {
                key_hash = &key_hash[1..];
            }

            let counter: u64 = seconds / element.period();
            let mut period_hash = hotp_hash::<D>(key_hash, counter);

            // calculate offset
            let offset: usize = match period_hash.last() {
                Some(result) => *result & 0xf,
                None => return Err(String::from("Invalid digest")),
            } as usize;

            period_hash[offset] &= 0x7f;

            // calculate code
            let code_bytes: [u8; 8] = match period_hash[offset..offset + 8].try_into() {
                Ok(x) => x,
                Err(_) => return Err(String::from("Invalid digest")),
            };

            let code = u64::from_be_bytes(code_bytes);

            Ok(to_yandex_string(code, element.digits() as usize))
        }
        None => Err(String::from("This element has not a yandex pin")),
    }
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
    use sha2::Sha256;

    use crate::otp::otp_element::OTPElement;

    use super::calculate_yandex_code;

    #[test]
    fn test_yandex() {
        let seconds: u64 = 1641559648;
        let otp_element = OTPElement::new(
            "6SB2IKNM6OBZPAVBVTOHDKS4FAAAAAAADFUTQMBTRY".to_string(),
            "".to_string(),
            "label".to_string(),
            8,
            "YANDEX".to_string(),
            "SHA256".to_string(),
            30,
            None,
            Some("5239".to_string()),
        );

        assert_eq!(
            calculate_yandex_code::<Sha256>(&otp_element, seconds).unwrap(),
            "umozdicq".to_uppercase()
        );
    }
}
