use argon2::{Config, ThreadMode, Variant, Version};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use chacha20poly1305::aead::{NewAead, Aead};
use data_encoding::BASE64;
use crate::encrypted_database::EncryptedDatabase;

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
    hash_length: XCHACHA20_POLY1305_KEY_LENGTH as u32
};

fn argon_derive_key(password_bytes: &[u8], salt: &[u8]) -> Result<Vec<u8>, String> {
    let config = KEY_DERIVATION_CONFIG;
    let hash = argon2::hash_raw(password_bytes, salt, &config);
    match hash {
        Ok(vec) => Ok(vec),
        Err(_e) => Err(String::from("Failed to derive encryption key")),
    }
}

pub fn encrypt_string(plaintext: String, password: &str) -> Result<String,String> {
    let mut salt: [u8;ARGON2ID_SALT_LENGTH] = [0;ARGON2ID_SALT_LENGTH];
    let mut nonce_bytes: [u8;XCHACHA20_POLY1305_NONCE_LENGTH] = [0;XCHACHA20_POLY1305_NONCE_LENGTH];
    if let Err(e) = getrandom::getrandom(&mut salt) {
        return Err(format!("Error during salt generation: {}",e));
    }
    if let Err(e) = getrandom::getrandom(&mut nonce_bytes) {
        return Err(format!("Error during nonce generation: {}",e));
    }

    let key: Vec<u8> = match argon_derive_key(password.as_bytes(),salt.as_slice()) {
        Ok(result) => result,
        Err(e) => return Err(e),
    };
    let wrapped_key = Key::from_slice(key.as_slice());

    let aead = XChaCha20Poly1305::new(wrapped_key);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let cipher_text = aead.encrypt(nonce, plaintext.as_bytes()).expect("Failed to encrypt");
    let encrypted_database = EncryptedDatabase::new(1,BASE64.encode(&nonce_bytes),BASE64.encode(&salt),BASE64.encode(&cipher_text));

    match serde_json::to_string(&encrypted_database) {
        Ok(result) => Ok(result),
        Err(e) => return Err(format!("Failed to serialize encrypted database: {}",e)),
    }
}

pub fn decrypt_string(encrypted_text: &str, password: &str) -> Result<String, String> {
    //encrypted text is an encrypted database json serialized object
    let encrypted_database: EncryptedDatabase = match serde_json::from_str(encrypted_text) {
        Ok(result) => result,
        Err(e) => {
            return Err(format!("Error during encrypted database deserialization: {}",e))
        },
    };
    let nonce = BASE64.decode(encrypted_database.nonce().as_bytes()).unwrap();
    let cipher_text = BASE64.decode(encrypted_database.cipher().as_bytes()).unwrap();
    let salt = BASE64.decode(encrypted_database.salt().as_bytes()).unwrap();

    let key: Vec<u8> = match argon_derive_key(password.as_bytes(),salt.as_slice()) {
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
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Error during UTF-8 string conversion: {}",e))
    }
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
                &encrypt_string(String::from("Secret data@#[]ò"), "pa$$w0rd").unwrap(),
                "pa$$w0rd",
            )
        );
    }
}