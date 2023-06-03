use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce}; // Or `Aes128Gcm`
use data_encoding::BASE64;
use hex::FromHex;
use serde::Deserialize;
use zeroize::Zeroize;

use crate::otp::otp_element::OTPElement;
use crate::utils;
use scrypt::{scrypt, Params};

use super::aegis::AegisDb;

#[derive(Deserialize)]
pub struct AegisEncryptedDatabase {
    //version: u32,
    header: AegisEncryptedHeader,
    db: String,
}

#[derive(Deserialize)]
struct AegisEncryptedHeader {
    slots: Vec<AegisEncryptedSlot>,
    params: AegisEncryptedParams,
}

#[derive(Deserialize)]
struct AegisEncryptedParams {
    nonce: String,
    tag: String,
}

#[derive(Deserialize)]
struct AegisEncryptedSlot {
    #[serde(rename = "type")]
    _type: u32,
    //uuid: String,
    key: String,
    key_params: AegisEncryptedParams,
    n: Option<u32>,
    r: Option<u32>,
    p: Option<u32>,
    salt: Option<String>,
    //repaired: Option<bool>,
}

impl TryFrom<AegisEncryptedDatabase> for Vec<OTPElement> {
    type Error = String;

    fn try_from(aegis_encrypted: AegisEncryptedDatabase) -> Result<Self, Self::Error> {
        let mut password = utils::password("Insert your Aegis password: ", 0);
        let master_key: Option<Vec<u8>> = get_master_key(&aegis_encrypted, &password);
        password.zeroize();

        match master_key {
            Some(mut master_key) => {
                let content = match BASE64.decode(aegis_encrypted.db.as_bytes()) {
                    Ok(result) => result,
                    Err(e) => return Err(format!("Error during base64 decoding: {e:?}")),
                };

                let key = GenericArray::clone_from_slice(master_key.as_slice());
                master_key.zeroize();
                let cipher = Aes256Gcm::new(&key);

                let decrypted_db = match cipher.decrypt(
                    Nonce::from_slice(
                        Vec::from_hex(&aegis_encrypted.header.params.nonce)
                            .expect("Failed to parse hex nonce")
                            .as_slice(),
                    ),
                    [
                        content,
                        Vec::from_hex(&aegis_encrypted.header.params.tag)
                            .expect("Failed to parse hex tag"),
                    ]
                    .concat()
                    .as_slice(),
                ) {
                    Ok(result) => result,
                    Err(e) => return Err(format!("Failed to derive master key: {e:?}")),
                };

                map_results(decrypted_db)
            }
            None => Err("Failed to derive master key".to_string()),
        }
    }
}

fn get_master_key(aegis_encrypted: &AegisEncryptedDatabase, password: &String) -> Option<Vec<u8>> {
    let mut master_key: Option<Vec<u8>> = None;
    for slot in aegis_encrypted
        .header
        .slots
        .iter()
        .filter(|item| item._type == 1)
    {
        match calc_master_key(slot, password.as_str()) {
            Ok(value) => {
                master_key = Some(value);
                break;
            }
            Err(e) => println!("{e}"),
        }
    }
    master_key
}

fn map_results(decrypted_db: Vec<u8>) -> Result<Vec<OTPElement>, String> {
    let json = match String::from_utf8(decrypted_db) {
        Ok(result) => result,
        Err(e) => return Err(format!("Failed to decode from utf-8 bytes: {e:?}")),
    };

    serde_json::from_str::<AegisDb>(json.as_str())
        .map_err(|e| e.to_string())
        .and_then(|e| e.try_into())
}

fn get_params(slot: &AegisEncryptedSlot) -> Result<Params, String> {
    let n = slot.n.unwrap();
    let p = slot.p.unwrap();
    let r = slot.r.unwrap();

    match Params::new(
        (n as f32).log2() as u8,
        r,
        p,
        scrypt::Params::RECOMMENDED_LEN,
    ) {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Error during scrypt params creation: {e:?}")),
    }
}

fn calc_master_key(slot: &AegisEncryptedSlot, password: &str) -> Result<Vec<u8>, String> {
    let salt = Vec::from_hex(slot.salt.as_ref().unwrap()).expect("Failed to parse hex salt");
    let mut output: [u8; 32] = [0; 32];
    let params = get_params(slot)?;

    if let Err(e) = scrypt(
        password.as_bytes(),
        salt.as_slice(),
        &params,
        output.as_mut_slice(),
    ) {
        return Err(format!("Error during scrypt key derivation: {e:?}"));
    }
    let key = GenericArray::clone_from_slice(output.as_slice());
    output.zeroize();

    let cipher = Aes256Gcm::new(&key);
    let cipher_text = [
        Vec::from_hex(&slot.key).expect("Failed to parse hex key"),
        Vec::from_hex(&slot.key_params.tag).expect("Failed to parse hex tag"),
    ]
    .concat();

    match cipher.decrypt(
        Nonce::from_slice(
            Vec::from_hex(&slot.key_params.nonce)
                .expect("Failed to parse hex nonce")
                .as_slice(),
        ),
        cipher_text.as_slice(),
    ) {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Failed to derive master key: {e:?}")),
    }
}
