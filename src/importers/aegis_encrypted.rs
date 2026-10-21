use aes_gcm::aead::{Aead, Nonce};
use aes_gcm::{Aes256Gcm, KeyInit}; // Or `Aes128Gcm`
use data_encoding::BASE64;
use hex::FromHex;
use serde::Deserialize;

use crate::otp::otp_element::OTPElement;
use crate::utils;
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

impl TryFrom<AegisEncryptedDatabase> for Vec<OTPElement> {
    type Error = String;
    fn try_from(database: AegisEncryptedDatabase) -> Result<Self, Self::Error> {
        let password = zeroize::Zeroizing::new(
            utils::password("Insert your Aegis password: ", 0).map_err(|e| e.to_string())?,
        );
        database.decrypt_with_password(&password)
    }
}

impl AegisEncryptedDatabase {
    pub fn decrypt_with_password(self, password: &str) -> Result<Vec<OTPElement>, String> {
        if self.header.slots.len() > 16 {
            return Err("Too many Aegis key slots".into());
        }
        let mut master_key = None;
        for slot in self.header.slots.iter().filter(|slot| slot.r#type == 1) {
            if let Ok(key) = calc_master_key(slot, password) {
                master_key = Some(zeroize::Zeroizing::new(key));
                break;
            }
        }
        let master_key =
            master_key.ok_or("Cannot decrypt Aegis backup: wrong password or invalid key slot")?;
        let content = BASE64
            .decode(self.db.as_bytes())
            .map_err(|_| "Invalid Aegis base64 data")?;
        let plaintext =
            zeroize::Zeroizing::new(decrypt(&master_key, &content, &self.header.params)?);
        let db: AegisDb = serde_json::from_slice(&plaintext)
            .map_err(|_| "Invalid decrypted Aegis account data")?;
        db.try_into()
    }
}

fn decrypt(key: &[u8], content: &[u8], params: &AegisEncryptedParams) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| "Invalid AES key length")?;
    let nonce_bytes = Vec::from_hex(&params.nonce).map_err(|_| "Invalid Aegis nonce encoding")?;
    let nonce = Nonce::<Aes256Gcm>::try_from(nonce_bytes.as_slice())
        .map_err(|_| "Invalid Aegis nonce length")?;
    let tag = Vec::from_hex(&params.tag).map_err(|_| "Invalid Aegis authentication tag")?;
    if tag.len() != 16 {
        return Err("Invalid Aegis authentication tag length".into());
    }
    let mut payload = content.to_vec();
    payload.extend_from_slice(&tag);
    cipher
        .decrypt(&nonce, payload.as_slice())
        .map_err(|_| "Aegis authentication failed".into())
}

fn calc_master_key(slot: &AegisEncryptedSlot, password: &str) -> Result<Vec<u8>, String> {
    let n = slot.n.ok_or("Missing scrypt N")?;
    let r = slot.r.ok_or("Missing scrypt r")?;
    let p = slot.p.ok_or("Missing scrypt p")?;
    if !n.is_power_of_two()
        || !(2..=262144).contains(&n)
        || r == 0
        || p == 0
        || u64::from(n) * u64::from(r) > 4_194_304
        || u64::from(n) * u64::from(r) * u64::from(p) > 8_388_608
    {
        return Err("Unsupported or excessive Aegis scrypt parameters".into());
    }
    let salt = Vec::from_hex(slot.salt.as_ref().ok_or("Missing scrypt salt")?)
        .map_err(|_| "Invalid scrypt salt")?;
    if !(16..=64).contains(&salt.len()) {
        return Err("Invalid scrypt salt length".into());
    }
    let params = Params::new(n.ilog2() as u8, r, p).map_err(|_| "Invalid scrypt parameters")?;
    let mut key = zeroize::Zeroizing::new([0u8; 32]);
    scrypt(password.as_bytes(), &salt, &params, key.as_mut())
        .map_err(|_| "Scrypt derivation failed")?;
    let encrypted = Vec::from_hex(&slot.key).map_err(|_| "Invalid encrypted Aegis key")?;
    decrypt(key.as_slice(), &encrypted, &slot.key_params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_slots_and_authentication_parameters_are_errors() {
        let input: AegisEncryptedDatabase = serde_json::from_value(serde_json::json!({
            "header":{"slots":[{"type":1,"key":"bad","key_params":{"nonce":"bad","tag":"bad"},"n":3,"r":8,"p":1,"salt":"bad"}],"params":{"nonce":"bad","tag":"bad"}},"db":"bad"
        })).unwrap();
        assert!(input.decrypt_with_password("password").is_err());
        for (nonce, tag) in [
            ("bad", "00"),
            ("00", "00"),
            ("000000000000000000000000", "00"),
        ] {
            assert!(
                decrypt(
                    &[0; 32],
                    b"",
                    &AegisEncryptedParams {
                        nonce: nonce.into(),
                        tag: tag.into()
                    }
                )
                .is_err()
            );
        }
    }
}
