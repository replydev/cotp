use std::{error, fmt};
use std::convert::TryInto;

use data_encoding::BASE64;
use sodiumoxide::crypto::pwhash;
use sodiumoxide::crypto::secretstream::{KEYBYTES, Stream, Tag};
use sodiumoxide::crypto::secretstream::xchacha20poly1305::{Header, Key};

const SIGNATURE: [u8; 4] = [0xC1, 0x0A, 0x4B, 0xED];

#[derive(Debug)]
struct CoreError {
    message: String,
}

impl CoreError {
    fn new(msg: &str) -> Self { CoreError { message: msg.to_string() } }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl error::Error for CoreError {}

fn argon_derive_key(key: &mut [u8; 32], password_bytes: &[u8], salt: &pwhash::argon2id13::Salt) -> Result<Key, String> {
    let result = pwhash::argon2id13::derive_key(key, password_bytes, salt,
                                                pwhash::argon2id13::OPSLIMIT_INTERACTIVE,
                                                pwhash::argon2id13::MEMLIMIT_INTERACTIVE);
    match result {
        Err(()) => Err(String::from("Failed to derive encryption key")),
        _ => Ok(Key(*key)),
    }
}

pub fn encrypt_string(plaintext: String, password: &str) -> String {
    let mut encrypted = String::new();
    encrypted.push_str(&BASE64.encode(&SIGNATURE));
    encrypted.push('|');
    let salt = pwhash::argon2id13::gen_salt();
    encrypted.push_str(&BASE64.encode(&salt.0));
    encrypted.push('|');
    let key = argon_derive_key(&mut [0u8; KEYBYTES], password.as_bytes(), &salt).unwrap();
    let (mut enc_stream, header) = Stream::init_push(&key).unwrap();

    encrypted.push_str(&BASE64.encode(&header.0));
    encrypted.push('|');

    let encrypted_string = enc_stream.push(plaintext.as_bytes(), None, Tag::Message).expect("Cannot encrypt");

    encrypted.push_str(&BASE64.encode(&encrypted_string));
    encrypted
}

pub fn decrypt_string(encrypted_text: &str, password: &str) -> Result<String, String> {
    let split = encrypted_text.split('|');
    let vec: Vec<&str> = split.collect();
    if vec.len() != 4 {
        return Err(String::from("Corrupted database file"));
    }
    let byte_salt = BASE64.decode(vec[1].as_bytes()).unwrap();
    let salt = pwhash::argon2id13::Salt(byte_vec_to_byte_array(byte_salt));
    let byte_header = BASE64.decode(vec[2].as_bytes()).unwrap();
    let header = Header(header_vec_to_header_array(byte_header));
    let cipher = BASE64.decode(vec[3].as_bytes()).unwrap();

    let mut key = [0u8; KEYBYTES];
    pwhash::argon2id13::derive_key(&mut key, password.as_bytes(), &salt,
                                   pwhash::argon2id13::OPSLIMIT_INTERACTIVE,
                                   pwhash::argon2id13::MEMLIMIT_INTERACTIVE)
        .map_err(|_| CoreError::new("Deriving key failed")).unwrap();
    let key = Key(key);

    let mut stream = Stream::init_pull(&header, &key)
        .map_err(|_| CoreError::new("init_pull failed")).unwrap();

    let (decrypted, _tag) = stream.pull(&cipher, None).unwrap_or((vec![0], Tag::Message));

    if decrypted == vec![0] {
        return Err(String::from("Wrong password"));
    }
    Ok(String::from_utf8(decrypted).unwrap())
}

fn byte_vec_to_byte_array(byte_vec: Vec<u8>) -> [u8; 16] {
    byte_vec.try_into()
        .unwrap_or_else(|v: Vec<u8>| panic!("Expected a Vec of length {} but it was {}", 16, v.len()))
}

fn header_vec_to_header_array(byte_vec: Vec<u8>) -> [u8; 24] {
    byte_vec.try_into()
        .unwrap_or_else(|v: Vec<u8>| panic!("Expected a Vec of length {} but it was {}", 24, v.len()))
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
            if password.len() >= minimum_password_length {
                break;
            }
        } else if password.len() >= minimum_password_length {
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
        assert_eq!(Ok(()), sodiumoxide::init());
        assert_eq!(
            String::from("Secret data@#[]ò"),
            decrypt_string(
                &mut encrypt_string(String::from("Secret data@#[]ò"), "pa$$w0rd"),
                "pa$$w0rd",
            ).unwrap()
        );
    }
}