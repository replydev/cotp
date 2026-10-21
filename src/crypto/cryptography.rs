use argon2::{Config, ThreadMode, Variant, Version};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use color_eyre::eyre::{ErrReport, eyre};
use data_encoding::BASE64;
use zeroize::{Zeroize, Zeroizing};

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
    thread_mode: ThreadMode::Sequential,
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
    let DecodedEnvelope {
        nonce,
        cipher_text,
        salt,
    } = decode_envelope(encrypted_text)?;

    let key = Zeroizing::new(argon_derive_key(password.as_bytes(), salt.as_slice())?);

    let aead = XChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| eyre!("Invalid encryption key length: {e}"))?;
    let nonce = XNonce::try_from(nonce.as_slice())
        .map_err(|_| eyre!("Invalid nonce length in encrypted database"))?;
    let decrypted = aead
        .decrypt(&nonce, cipher_text.as_slice())
        .map_err(|_| eyre!("Wrong password"))?;
    let from_utf8 = String::from_utf8(decrypted).map_err(|error| {
        error.into_bytes().zeroize();
        eyre!("Decrypted vault is not UTF-8")
    })?;
    Ok((from_utf8, key.to_vec(), salt))
}

struct DecodedEnvelope {
    nonce: Vec<u8>,
    cipher_text: Vec<u8>,
    salt: Vec<u8>,
}

/// Check the encrypted container without a password. Authentication still requires decrypting it.
pub(crate) fn validate_envelope(bytes: &[u8]) -> color_eyre::Result<()> {
    decode_envelope(std::str::from_utf8(bytes)?)?;
    Ok(())
}

fn decode_envelope(encrypted_text: &str) -> color_eyre::Result<DecodedEnvelope> {
    let encrypted_database: EncryptedDatabase = serde_json::from_str(encrypted_text)
        .map_err(|e| eyre!("Error during encrypted database deserialization: {e}"))?;
    if encrypted_database.version() != 1 {
        return Err(eyre!("Unsupported encrypted database version"));
    }
    let nonce = BASE64
        .decode(encrypted_database.nonce().as_bytes())
        .map_err(|_| eyre!("Invalid Base64 nonce"))?;
    let cipher_text = BASE64
        .decode(encrypted_database.cipher().as_bytes())
        .map_err(|_| eyre!("Invalid Base64 ciphertext"))?;
    let salt = BASE64
        .decode(encrypted_database.salt().as_bytes())
        .map_err(|_| eyre!("Invalid Base64 salt"))?;
    if nonce.len() != XCHACHA20_POLY1305_NONCE_LENGTH || salt.len() != ARGON2ID_SALT_LENGTH {
        return Err(eyre!("Invalid nonce or salt length"));
    }

    if cipher_text.len() < 16 {
        return Err(eyre!("Truncated encrypted vault ciphertext"));
    }
    Ok(DecodedEnvelope {
        nonce,
        cipher_text,
        salt,
    })
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

#[cfg(test)]
mod malformed_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn malformed_json_never_panics(input in ".{0,2048}") {
            let _ = decrypt_string(&input, "test password");
        }
        #[test]
        fn malformed_envelopes_never_panic(nonce in ".{0,40}", salt in ".{0,30}", cipher in ".{0,100}") {
            let input = serde_json::json!({"version":1,"nonce":nonce,"salt":salt,"cipher":cipher});
            let _ = decrypt_string(&input.to_string(), "test password");
        }
    }

    #[test]
    fn tampering_and_wrong_password_fail_authentication() {
        let salt = gen_salt().unwrap();
        let key = argon_derive_key(b"correct password", &salt).unwrap();
        let encrypted = encrypt_string_with_key("private vault", &key, &salt).unwrap();
        let serialized = serde_json::to_string(&encrypted).unwrap();
        assert!(decrypt_string(&serialized, "wrong password").is_err());
        let mut value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        let mut cipher = BASE64
            .decode(value["cipher"].as_str().unwrap().as_bytes())
            .unwrap();
        cipher[0] ^= 1;
        value["cipher"] = BASE64.encode(&cipher).into();
        assert!(decrypt_string(&value.to_string(), "correct password").is_err());
        value["version"] = 65535.into();
        assert!(decrypt_string(&value.to_string(), "correct password").is_err());
    }
}

#[cfg(test)]
mod envelope_tests {
    use super::*;

    #[test]
    fn structural_validation_rejects_unsupported_and_truncated_containers() {
        let valid = serde_json::json!({"version":1,"nonce":BASE64.encode(&[0;24]),"salt":BASE64.encode(&[0;16]),"cipher":BASE64.encode(&[0;16])});
        assert!(validate_envelope(valid.to_string().as_bytes()).is_ok());
        for (field, value) in [
            ("version", serde_json::json!(2)),
            ("nonce", serde_json::json!(BASE64.encode(&[0; 23]))),
            ("salt", serde_json::json!(BASE64.encode(&[0; 15]))),
            ("cipher", serde_json::json!(BASE64.encode(&[0; 15]))),
            ("cipher", serde_json::json!("not base64!")),
        ] {
            let mut corrupt = valid.clone();
            corrupt[field] = value;
            assert!(
                validate_envelope(corrupt.to_string().as_bytes()).is_err(),
                "{field}"
            );
        }
    }

    #[test]
    fn authenticated_invalid_utf8_returns_a_sanitized_error() {
        let salt = [1; 16];
        let key = Zeroizing::new(argon_derive_key(b"password", &salt).unwrap());
        let nonce = XNonce::from([2; 24]);
        let aead = XChaCha20Poly1305::new_from_slice(&key).unwrap();
        let cipher = aead.encrypt(&nonce, &[0xff, 0xfe][..]).unwrap();
        let envelope = EncryptedDatabase::new(
            1,
            BASE64.encode(&nonce),
            BASE64.encode(&salt),
            BASE64.encode(&cipher),
        );
        let error =
            decrypt_string(&serde_json::to_string(&envelope).unwrap(), "password").unwrap_err();
        assert_eq!(error.to_string(), "Decrypted vault is not UTF-8");
    }
}
