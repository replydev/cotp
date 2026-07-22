use aes_gcm::aead::{Aead, Nonce};
use aes_gcm::{Aes256Gcm, KeyInit}; // Or `Aes128Gcm`
use data_encoding::{BASE64, DecodeError, HEXLOWER_PERMISSIVE};
use serde::Deserialize;
use zeroize::Zeroize;

use crate::otp::otp_element::OTPElement;
use scrypt::{Params, scrypt};

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
    r#type: u32,
    //uuid: String,
    key: String,
    key_params: AegisEncryptedParams,
    n: Option<u32>,
    r: Option<u32>,
    p: Option<u32>,
    salt: Option<String>,
    //repaired: Option<bool>,
}

impl AegisEncryptedDatabase {
    /// Decrypts the backup contents using the given password and maps the
    /// entries into `OTPElement` values.
    pub fn decrypt(self, password: &str) -> Result<Vec<OTPElement>, String> {
        let master_key: Option<Vec<u8>> = get_master_key(&self, password);

        match master_key {
            Some(mut master_key) => {
                let content = BASE64
                    .decode(self.db.as_bytes())
                    .map_err(|e| format!("Error during base64 decoding: {e:?}"))?;

                let cipher = Aes256Gcm::new_from_slice(master_key.as_slice())
                    .map_err(|e| format!("Invalid master key length: {e:?}"))?;
                master_key.zeroize();

                let nonce_bytes = decode_hex(&self.header.params.nonce)
                    .map_err(|e| format!("Failed to parse hex nonce: {e:?}"))?;
                let nonce = Nonce::<Aes256Gcm>::try_from(nonce_bytes.as_slice())
                    .map_err(|e| format!("Invalid nonce length: {e:?}"))?;

                let payload = [
                    content,
                    decode_hex(&self.header.params.tag)
                        .map_err(|e| format!("Failed to parse hex tag: {e:?}"))?,
                ]
                .concat();

                let decrypted_db = cipher
                    .decrypt(&nonce, payload.as_slice())
                    .map_err(|e| format!("Failed to derive master key: {e:?}"))?;

                map_results(decrypted_db)
            }
            None => Err("Failed to derive master key".to_string()),
        }
    }
}

/// Decodes a hex string, accepting both lower- and uppercase digits like the
/// previously used `hex` crate did (Aegis itself writes lowercase).
fn decode_hex(input: &str) -> Result<Vec<u8>, DecodeError> {
    HEXLOWER_PERMISSIVE.decode(input.as_bytes())
}

fn get_master_key(aegis_encrypted: &AegisEncryptedDatabase, password: &str) -> Option<Vec<u8>> {
    let mut master_key: Option<Vec<u8>> = None;
    for slot in aegis_encrypted
        .header
        .slots
        .iter()
        .filter(|item| item.r#type == 1)
    {
        match calc_master_key(slot, password) {
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
    let mut json = match String::from_utf8(decrypted_db) {
        Ok(json) => json,
        Err(e) => {
            let error = format!("Failed to decode from utf-8 bytes: {:?}", e.utf8_error());
            e.into_bytes().zeroize();
            return Err(error);
        }
    };

    let result = serde_json::from_str::<AegisDb>(json.as_str())
        .map_err(|e| e.to_string())
        .and_then(TryInto::try_into);
    json.zeroize();
    result
}

fn get_params(slot: &AegisEncryptedSlot) -> Result<Params, String> {
    let n = slot.n.ok_or("Missing scrypt parameter n in backup slot")?;
    let p = slot.p.ok_or("Missing scrypt parameter p in backup slot")?;
    let r = slot.r.ok_or("Missing scrypt parameter r in backup slot")?;

    if !n.is_power_of_two() {
        return Err(format!(
            "Invalid scrypt parameter n: {n} is not a power of two"
        ));
    }

    Params::new(n.trailing_zeros() as u8, r, p)
        .map_err(|e| format!("Error during scrypt params creation: {e:?}"))
}

fn calc_master_key(slot: &AegisEncryptedSlot, password: &str) -> Result<Vec<u8>, String> {
    let salt_hex = slot.salt.as_ref().ok_or("Missing salt in backup slot")?;
    let salt = decode_hex(salt_hex).map_err(|e| format!("Failed to parse hex salt: {e:?}"))?;
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
    let cipher = Aes256Gcm::new_from_slice(output.as_slice())
        .map_err(|e| format!("Invalid derived key length: {e:?}"))?;
    output.zeroize();

    let cipher_text = [
        decode_hex(&slot.key).map_err(|e| format!("Failed to parse hex key: {e:?}"))?,
        decode_hex(&slot.key_params.tag).map_err(|e| format!("Failed to parse hex tag: {e:?}"))?,
    ]
    .concat();

    let nonce_bytes = decode_hex(&slot.key_params.nonce)
        .map_err(|e| format!("Failed to parse hex nonce: {e:?}"))?;
    let nonce = Nonce::<Aes256Gcm>::try_from(nonce_bytes.as_slice())
        .map_err(|e| format!("Invalid nonce length: {e:?}"))?;

    cipher
        .decrypt(&nonce, cipher_text.as_slice())
        .map_err(|e| format!("Failed to derive master key: {e:?}"))
}

#[cfg(test)]
mod tests {
    use super::{AegisEncryptedDatabase, AegisEncryptedSlot, calc_master_key, get_params};

    fn slot_from_json(json: &str) -> AegisEncryptedSlot {
        serde_json::from_str(json).expect("Invalid test slot JSON")
    }

    #[test]
    fn decrypt_with_no_usable_slot_returns_error() {
        // The only type-1 slot is malformed (missing scrypt parameters), so no
        // master key can be derived and decrypt must fail gracefully.
        let database: AegisEncryptedDatabase = serde_json::from_str(
            r#"{
                "version": 1,
                "header": {
                    "slots": [
                        {
                            "type": 1,
                            "key": "00",
                            "key_params": {"nonce": "00", "tag": "00"},
                            "salt": "00"
                        }
                    ],
                    "params": {"nonce": "00", "tag": "00"}
                },
                "db": "AAAA"
            }"#,
        )
        .expect("Invalid test database JSON");

        let result = database.decrypt("password");
        assert_eq!(Err("Failed to derive master key".to_string()), result);
    }

    #[test]
    fn missing_scrypt_params_return_error() {
        let slot = slot_from_json(
            r#"{
                "type": 1,
                "key": "00",
                "key_params": {"nonce": "00", "tag": "00"},
                "salt": "00"
            }"#,
        );

        let result = calc_master_key(&slot, "password");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing scrypt parameter"));
    }

    #[test]
    fn missing_salt_returns_error() {
        let slot = slot_from_json(
            r#"{
                "type": 1,
                "key": "00",
                "key_params": {"nonce": "00", "tag": "00"},
                "n": 2,
                "r": 8,
                "p": 1
            }"#,
        );

        let result = calc_master_key(&slot, "password");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing salt"));
    }

    #[test]
    fn non_power_of_two_n_returns_error() {
        let slot = slot_from_json(
            r#"{
                "type": 1,
                "key": "00",
                "key_params": {"nonce": "00", "tag": "00"},
                "n": 15000,
                "r": 8,
                "p": 1,
                "salt": "00"
            }"#,
        );

        let result = get_params(&slot);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a power of two"));
    }

    #[test]
    fn non_hex_salt_returns_error() {
        let slot = slot_from_json(
            r#"{
                "type": 1,
                "key": "00",
                "key_params": {"nonce": "00", "tag": "00"},
                "n": 2,
                "r": 8,
                "p": 1,
                "salt": "not-hex"
            }"#,
        );

        let result = calc_master_key(&slot, "password");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse hex salt"));
    }

    #[test]
    fn non_hex_key_returns_error() {
        let slot = slot_from_json(
            r#"{
                "type": 1,
                "key": "zz",
                "key_params": {"nonce": "00", "tag": "00"},
                "n": 2,
                "r": 8,
                "p": 1,
                "salt": "00"
            }"#,
        );

        let result = calc_master_key(&slot, "password");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse hex key"));
    }
}
