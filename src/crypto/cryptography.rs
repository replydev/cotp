use argon2::{Config, Variant, Version};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce};
use color_eyre::eyre::{eyre, ErrReport};
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
    secret: &[],
    ad: &[],
    hash_length: XCHACHA20_POLY1305_KEY_LENGTH as u32,
};

pub fn argon_derive_key(password_bytes: &[u8], salt: &[u8]) -> color_eyre::Result<Vec<u8>> {
    argon2::hash_raw(password_bytes, salt, &KEY_DERIVATION_CONFIG).map_err(ErrReport::from)
}

pub fn gen_salt() -> color_eyre::Result<[u8; ARGON2ID_SALT_LENGTH]> {
    let mut salt: [u8; ARGON2ID_SALT_LENGTH] = [0; ARGON2ID_SALT_LENGTH];
    getrandom::fill(&mut salt).map_err(|e| eyre!(e))?;
    Ok(salt)
}

pub fn encrypt_string_with_key(
    plain_text: &str,
    key: &Vec<u8>,
    salt: &[u8],
) -> color_eyre::Result<EncryptedDatabase> {
    let wrapped_key = Key::from_slice(key.as_slice());

    let aead = XChaCha20Poly1305::new(wrapped_key);
    let mut nonce_bytes: [u8; XCHACHA20_POLY1305_NONCE_LENGTH] =
        [0; XCHACHA20_POLY1305_NONCE_LENGTH];

    getrandom::fill(&mut nonce_bytes).map_err(|e| eyre!(e))?;

    let nonce = XNonce::from_slice(&nonce_bytes);
    let cipher_text = aead
        .encrypt(nonce, plain_text.as_bytes())
        .map_err(|e| eyre!("Error during encryption: {e}"))?;
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
) -> color_eyre::Result<(String, Vec<u8>, Vec<u8>)> {
    //encrypted text is an encrypted database json serialized object
    let encrypted_database: EncryptedDatabase = serde_json::from_str(encrypted_text)
        .map_err(|e| eyre!("Error during encrypted database deserialization: {e}"))?;
    let nonce = BASE64
        .decode(encrypted_database.nonce().as_bytes())
        .expect("Cannot decode Base64 nonce");
    let cipher_text = BASE64
        .decode(encrypted_database.cipher().as_bytes())
        .expect("Cannot decode Base64 cipher");
    let salt = BASE64.decode(encrypted_database.salt().as_bytes()).unwrap();

    let key: Vec<u8> = argon_derive_key(password.as_bytes(), salt.as_slice())?;

    let wrapped_key = Key::from_slice(&key);

    let aead = XChaCha20Poly1305::new(wrapped_key);
    let nonce = XNonce::from_slice(nonce.as_slice());
    let decrypted = aead
        .decrypt(nonce, cipher_text.as_slice())
        .map_err(|_| eyre!("Wrong password"))?;
    let from_utf8 = String::from_utf8(decrypted).map_err(ErrReport::from)?;
    Ok((from_utf8, key, salt))
}

#[cfg(test)]
mod tests {
    use crate::crypto::cryptography::{argon_derive_key, gen_salt};

    use super::{decrypt_string, encrypt_string_with_key};

    #[test]
    fn test_encryption() {
        let salt = gen_salt().unwrap();
        let key = argon_derive_key(b"pa$$w0rd", salt.as_ref()).unwrap();
        let encrypted = encrypt_string_with_key("Secret data@#[]ò", &key, salt.as_ref()).unwrap();
        let (decrypted, _key, _salt) =
            decrypt_string(&serde_json::to_string(&encrypted).unwrap(), "pa$$w0rd").unwrap();
        assert_eq!(String::from("Secret data@#[]ò"), decrypted);
    }
}
