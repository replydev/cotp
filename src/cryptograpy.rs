use std::{error, fmt};
use std::convert::TryInto;
use sodiumoxide::crypto::pwhash;
use sodiumoxide::crypto::secretstream::{Stream, Tag, KEYBYTES};
use sodiumoxide::crypto::secretstream::xchacha20poly1305::{Header, Key};

const SIGNATURE: [u8;4] = [0xC1, 0x0A, 0x4B, 0xED];

#[derive(Debug)]
struct CoreError {
    message: String,
}

impl CoreError {
    fn new(msg: &str) -> Self { CoreError{message: msg.to_string()} }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl error::Error for CoreError {}

pub fn encrypt_string(plaintext: &mut String,password: &str) -> String {
    let mut encrypted = String::new();
    encrypted.push_str(&base64::encode(SIGNATURE));
    encrypted.push('|');
    let salt = pwhash::gen_salt();
    encrypted.push_str(&base64::encode(salt.0));
    encrypted.push('|');
    let mut key = [0u8; KEYBYTES];
    pwhash::derive_key(&mut key, password.as_bytes(), &salt,
        pwhash::OPSLIMIT_INTERACTIVE,
        pwhash::MEMLIMIT_INTERACTIVE).unwrap();
    let key = Key(key);

    let (mut enc_stream, header) = Stream::init_push(&key).unwrap();

    encrypted.push_str(&base64::encode(header.0));
    encrypted.push('|');

    let encrypted_string = enc_stream.push(plaintext.as_bytes(), None, Tag::Message).expect("Cannot encrypt");

    encrypted.push_str(&base64::encode(encrypted_string));
    encrypted
}

pub fn decrypt_string(encrypted_text: &mut str,password: &str) -> Result<String, String> {
    let split = encrypted_text.split('|');
    let vec: Vec<&str> = split.collect();
    let byte_salt = base64::decode(vec[1]).unwrap();
    let salt = pwhash::Salt(byte_vec_to_byte_array(byte_salt));
    let byte_header = base64::decode(vec[2]).unwrap();
    let header = Header(header_vec_to_header_array(byte_header));
    let cipher = base64::decode(vec[3]).unwrap();

    let mut key = [0u8; KEYBYTES];
    pwhash::derive_key(&mut key, password.as_bytes(), &salt,
        pwhash::OPSLIMIT_INTERACTIVE,
        pwhash::MEMLIMIT_INTERACTIVE)
        .map_err(|_| CoreError::new("Deriving key failed")).unwrap();
    let key = Key(key);

    let mut stream = Stream::init_pull(&header, &key)
        .map_err(|_| CoreError::new("init_pull failed")).unwrap();

    let (decrypted, _tag) = stream.pull(&cipher, None).unwrap_or((vec![0],Tag::Message));

    if decrypted == vec![0]{
        return Err(String::from("Wrong password"));
    }
    Ok(String::from_utf8(decrypted).unwrap())
}

fn byte_vec_to_byte_array(byte_vec: Vec<u8>) -> [u8;32]{
    byte_vec.try_into()
        .unwrap_or_else(|v: Vec<u8>| panic!("Expected a Vec of length {} but it was {}", 32, v.len()))
}

fn header_vec_to_header_array(byte_vec: Vec<u8>) -> [u8;24]{
    byte_vec.try_into()
        .unwrap_or_else(|v: Vec<u8>| panic!("Expected a Vec of length {} but it was {}", 24, v.len()))
}

pub fn prompt_for_passwords(message: &str) -> String{
    let mut password;
    loop{
        password = rpassword::prompt_password_stdout(message).unwrap();
        if password.len() >= 8 {
            break;
        }
    }
    password
}