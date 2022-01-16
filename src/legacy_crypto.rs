use std::{error, fmt};
use std::convert::TryInto;

use data_encoding::BASE64;
use sodiumoxide::crypto::pwhash;
use sodiumoxide::crypto::secretstream::{KEYBYTES, Stream, Tag};
use sodiumoxide::crypto::secretstream::xchacha20poly1305::{Header, Key};

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

pub fn decrypt_string(encrypted_text: &str, password: &str) -> Result<String, String> {
    let split = encrypted_text.split('|');
    let vec: Vec<&str> = split.collect();
    if vec.len() != 4 {
        return Err(String::from("Corrupted database file"));
    }
    let byte_salt = BASE64.decode(vec[1].as_bytes()).unwrap();
    let salt = pwhash::argon2id13::Salt(vec_to_arr(byte_salt));
    let byte_header = BASE64.decode(vec[2].as_bytes()).unwrap();
    let header = Header(vec_to_arr(byte_header));
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

fn vec_to_arr<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}