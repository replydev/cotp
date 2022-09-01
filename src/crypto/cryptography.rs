use argon2::{Config, ThreadMode, Variant, Version};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce};
use data_encoding::BASE64;

use super::encrypted_database::EncryptedDatabase;

const ARGON2ID_SALT_LENGTH: usize = 16;
const XCHACHA20_POLY1305_NONCE_LENGTH: usize = 24;
const XCHACHA20_POLY1305_KEY_LENGTH: usize = 32;
const KEY_DERIVATION_CONFIG: Config = Config {
    variant: Variant::Argon2id,
    version: Version::Version13,
    mem_cost: 32768,
    time_cost: 4,
    lanes: 4,
    thread_mode: ThreadMode::Parallel,
    secret: &[],
    ad: &[],
    hash_length: XCHACHA20_POLY1305_KEY_LENGTH as u32,
};

pub fn argon_derive_key(password_bytes: &[u8], salt: &[u8]) -> Result<Vec<u8>, String> {
    let config = KEY_DERIVATION_CONFIG;
    let hash = argon2::hash_raw(password_bytes, salt, &config);
    match hash {
        Ok(vec) => Ok(vec),
        Err(_e) => Err(String::from("Failed to derive encryption key")),
    }
}

pub fn gen_salt() -> Result<[u8; ARGON2ID_SALT_LENGTH], String> {
    let mut salt: [u8; ARGON2ID_SALT_LENGTH] = [0; ARGON2ID_SALT_LENGTH];
    if let Err(e) = getrandom::getrandom(&mut salt) {
        //return Err(format!("Error during salt generation: {}", e));
        return Err(format!("Error during salt generation: {}", e));
    }
    Ok(salt)
}

pub fn encrypt_string_with_key(
    plain_text: String,
    key: &Vec<u8>,
    salt: &[u8],
) -> Result<EncryptedDatabase, String> {
    let wrapped_key = Key::from_slice(key.as_slice());

    let aead = XChaCha20Poly1305::new(wrapped_key);
    let mut nonce_bytes: [u8; XCHACHA20_POLY1305_NONCE_LENGTH] =
        [0; XCHACHA20_POLY1305_NONCE_LENGTH];
    if let Err(e) = getrandom::getrandom(&mut nonce_bytes) {
        return Err(format!("Error during nonce generation: {}", e));
    }
    let nonce = XNonce::from_slice(&nonce_bytes);
    let cipher_text = aead
        .encrypt(nonce, plain_text.as_bytes())
        .expect("Failed to encrypt");
    Ok(EncryptedDatabase::new(
        1,
        BASE64.encode(&nonce_bytes),
        BASE64.encode(salt),
        BASE64.encode(&cipher_text),
    ))
}

pub fn decrypt_string(
    encrypted_text: &str,
    password: &str,
) -> Result<(String, Vec<u8>, Vec<u8>), String> {
    //encrypted text is an encrypted database json serialized object
    let encrypted_database: EncryptedDatabase = match serde_json::from_str(encrypted_text) {
        Ok(result) => result,
        Err(e) => {
            return Err(format!(
                "Error during encrypted database deserialization: {}",
                e
            ))
        }
    };
    let nonce = BASE64
        .decode(encrypted_database.nonce().as_bytes())
        .unwrap();
    let cipher_text = BASE64
        .decode(encrypted_database.cipher().as_bytes())
        .unwrap();
    let salt = BASE64.decode(encrypted_database.salt().as_bytes()).unwrap();

    let key: Vec<u8> = match argon_derive_key(password.as_bytes(), salt.as_slice()) {
        Ok(result) => result,
        Err(e) => return Err(e),
    };

    let wrapped_key = Key::from_slice(&key);

    let aead = XChaCha20Poly1305::new(wrapped_key);
    let nonce = XNonce::from_slice(nonce.as_slice());
    let decrypted = match aead.decrypt(nonce, cipher_text.as_slice()) {
        Ok(result) => result,
        Err(_e) => return Err(String::from("Wrong password")),
    };
    match String::from_utf8(decrypted) {
        Ok(result) => Ok((result, key, salt)),
        Err(e) => Err(format!("Error during UTF-8 string conversion: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::cryptography::{argon_derive_key, gen_salt};

    use super::{decrypt_string, encrypt_string_with_key};

    #[test]
    fn test_encryption() {
        let salt = gen_salt().unwrap();
        let key = argon_derive_key(b"pa$$w0rd", salt.as_ref()).unwrap();
        let encrypted =
            encrypt_string_with_key(String::from("Secret data@#[]ò"), &key, salt.as_ref()).unwrap();
        let (decrypted, _key, _salt) =
            decrypt_string(&serde_json::to_string(&encrypted).unwrap(), "pa$$w0rd").unwrap();
        assert_eq!(String::from("Secret data@#[]ò"), decrypted);
    }
}
