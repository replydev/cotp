use std::convert::TryInto;

use argon2::Config;
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use chacha20poly1305::aead::{NewAead, Aead};
use data_encoding::BASE64;
use crate::encrypted_database::EncryptedDatabase;

pub const ARGON2ID_SALT_LENGTH: usize = 16;
pub const XCHACHA20_POLY1305_NONCE_LENGTH: usize = 24;
pub const XCHACHA20_POLY1305_KEY_LENGTH: usize = 32;

fn argon_derive_key(key: &mut [u8;XCHACHA20_POLY1305_KEY_LENGTH], password_bytes: &[u8], salt: &[u8]) -> Result<(), String> {
    let config = Config::default();
    let hash = argon2::hash_raw(password_bytes, salt, &config);
    match hash {
        Ok(vec) => {
            key.clone_from(&vec_to_arr(vec));
            Ok(())
        },
        Err(_e) => Err(String::from("Failed to derive encryption key")),
    }
}

pub fn encrypt_string(plaintext: String, password: &str) -> String {
    let mut salt: [u8;ARGON2ID_SALT_LENGTH] = [0;ARGON2ID_SALT_LENGTH];
    let mut nonce_bytes: [u8;XCHACHA20_POLY1305_NONCE_LENGTH] = [0;XCHACHA20_POLY1305_NONCE_LENGTH];
    match getrandom::getrandom(&mut salt) {
        Err(_e) => panic!("Error during salt generation"),
        _ => {}
    }
    match getrandom::getrandom(&mut nonce_bytes) {
        Err(_e) => panic!("Error during nonce generation"),
        _ => {}
    }

    let mut key: [u8;XCHACHA20_POLY1305_KEY_LENGTH] = [0;XCHACHA20_POLY1305_KEY_LENGTH];
    match argon_derive_key(&mut key, password.as_bytes(), &salt) {
        Err(e) => panic!("{}",e),
        _ => {}
    }
    let wrapped_key = Key::from_slice(&key);

    let aead = XChaCha20Poly1305::new(wrapped_key);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let cipher_text = aead.encrypt(nonce, plaintext.as_bytes()).expect("Failed to encrypt");
    let encrypted_database = EncryptedDatabase::new(1,BASE64.encode(&nonce_bytes),BASE64.encode(&salt),BASE64.encode(&cipher_text));

    match serde_json::to_string(&encrypted_database) {
        Ok(result) => result,
        Err(e) => panic!("Failed to serialize encrypted database: {}",e),
    }
}

pub fn decrypt_string(encrypted_text: &str, password: &str) -> Result<String, String> {
    //encrypted text is an encrypted database json serialized object
    let encrypted_database: EncryptedDatabase = match serde_json::from_str(encrypted_text) {
        Ok(result) => result,
        Err(e) => return Err(format!("Error during encrypted database deserialization: {}",e)),
    };
    let nonce = BASE64.decode(encrypted_database.nonce().as_bytes()).unwrap();
    let cipher_text = BASE64.decode(encrypted_database.cipher().as_bytes()).unwrap();
    let salt = BASE64.decode(encrypted_database.salt().as_bytes()).unwrap();

    let mut key: [u8;XCHACHA20_POLY1305_KEY_LENGTH] = [0;XCHACHA20_POLY1305_KEY_LENGTH];
    match argon_derive_key(&mut key, password.as_bytes(), salt.as_slice()) {
        Err(e) => panic!("{}",e),
        _ => {}
    }

    let wrapped_key = Key::from_slice(&key);

    let aead = XChaCha20Poly1305::new(wrapped_key);
    let nonce = XNonce::from_slice(nonce.as_slice());
    let decrypted = aead.decrypt(nonce, cipher_text.as_slice()).expect("Failed to decrypt");
    match String::from_utf8(decrypted) {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Error during UTF-8 string conversion: {}",e))
    }
}

fn vec_to_arr<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

pub fn prompt_for_passwords(message: &str, minimum_password_length: usize, verify: bool) -> String {
    let mut password;
    loop {
        password = rpassword::prompt_password_stdout(message).unwrap();
        if verify {
            let verify_password = rpassword::prompt_password_stdout("Retype the same password: ").unwrap();
            if password != verify_password {
                println!("Passwords do not match");
                continue;
            }
            if password.chars().count() >= minimum_password_length {
                break;
            }
        } else if password.chars().count() >= minimum_password_length {
            break;
        }
        println!("Please insert a password with at least {} digits.", minimum_password_length);
    }
    password
}


#[cfg(test)]
mod tests {
    use super::{decrypt_string, encrypt_string};

    #[test]
    fn test_encryption() {
        assert_eq!(
            Ok(String::from("Secret data@#[]ò")),
            decrypt_string(
                &encrypt_string(String::from("Secret data@#[]ò"), "pa$$w0rd"),
                "pa$$w0rd",
            )
        );
    }
}