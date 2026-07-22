use argon2::{Config, ThreadMode, Variant, Version};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use color_eyre::eyre::{ErrReport, eyre};
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
    // Parallel only changes the execution strategy: the derived key depends on
    // the lane count (4), not on how many threads compute the lanes, so
    // existing databases decrypt identically (see
    // test_derived_key_unchanged_by_thread_mode).
    thread_mode: ThreadMode::Parallel,
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
    let aead = XChaCha20Poly1305::new_from_slice(key.as_slice())
        .map_err(|e| eyre!("Invalid encryption key length: {e}"))?;
    let mut nonce_bytes: [u8; XCHACHA20_POLY1305_NONCE_LENGTH] =
        [0; XCHACHA20_POLY1305_NONCE_LENGTH];

    getrandom::fill(&mut nonce_bytes).map_err(|e| eyre!(e))?;

    let nonce = XNonce::from(nonce_bytes);
    let cipher_text = aead
        .encrypt(&nonce, plain_text.as_bytes())
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
        .map_err(|e| eyre!("database file is corrupted: cannot decode Base64 nonce: {e}"))?;
    let cipher_text = BASE64
        .decode(encrypted_database.cipher().as_bytes())
        .map_err(|e| eyre!("database file is corrupted: cannot decode Base64 cipher: {e}"))?;
    let salt = BASE64
        .decode(encrypted_database.salt().as_bytes())
        .map_err(|e| eyre!("database file is corrupted: cannot decode Base64 salt: {e}"))?;

    let key: Vec<u8> = argon_derive_key(password.as_bytes(), salt.as_slice())?;

    let aead = XChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| eyre!("Invalid encryption key length: {e}"))?;
    let nonce = XNonce::try_from(nonce.as_slice())
        .map_err(|_| eyre!("Invalid nonce length in encrypted database"))?;
    let decrypted = aead
        .decrypt(&nonce, cipher_text.as_slice())
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

    /// The expected value below was captured from the previous
    /// `ThreadMode::Sequential` configuration. It must never change: the lane
    /// count (4) is the Argon2 hash parameter, while the thread mode is only
    /// the execution strategy, so switching to `ThreadMode::Parallel` must
    /// derive the exact same key and keep existing databases decryptable.
    #[test]
    fn test_derived_key_unchanged_by_thread_mode() {
        let key = argon_derive_key(b"pa$$w0rd", b"0123456789abcdef").unwrap();
        let hex: String = key.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "1bae31857cc03a6fbb54f463a991e09e3294bdf3b8c44e4ddcec4ec1d7d6a4a7"
        );
    }

    #[test]
    fn test_decrypt_invalid_base64_nonce_returns_error() {
        let corrupted =
            r#"{"version":1,"nonce":"!!!not-base64!!!","salt":"c2FsdA==","cipher":"Y2lwaGVy"}"#;
        let result = decrypt_string(corrupted, "pa$$w0rd");
        let error = result.err().expect("corrupted nonce must not panic");
        assert!(error.to_string().contains("database file is corrupted"));
        assert!(error.to_string().contains("nonce"));
    }

    #[test]
    fn test_decrypt_invalid_base64_cipher_returns_error() {
        let corrupted =
            r#"{"version":1,"nonce":"bm9uY2U=","salt":"c2FsdA==","cipher":"!!!not-base64!!!"}"#;
        let result = decrypt_string(corrupted, "pa$$w0rd");
        let error = result.err().expect("corrupted cipher must not panic");
        assert!(error.to_string().contains("database file is corrupted"));
        assert!(error.to_string().contains("cipher"));
    }

    #[test]
    fn test_decrypt_invalid_base64_salt_returns_error() {
        let corrupted =
            r#"{"version":1,"nonce":"bm9uY2U=","salt":"!!!not-base64!!!","cipher":"Y2lwaGVy"}"#;
        let result = decrypt_string(corrupted, "pa$$w0rd");
        let error = result.err().expect("corrupted salt must not panic");
        assert!(error.to_string().contains("database file is corrupted"));
        assert!(error.to_string().contains("salt"));
    }
}
