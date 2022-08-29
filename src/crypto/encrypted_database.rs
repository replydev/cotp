use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EncryptedDatabase {
    version: u16,
    nonce: String,
    salt: String,
    cipher: String,
}

impl EncryptedDatabase {
    pub fn new(version: u16, nonce: String, salt: String, cipher: String) -> EncryptedDatabase {
        EncryptedDatabase {
            version,
            nonce,
            salt,
            cipher,
        }
    }

    /*
    pub fn version(&self) -> u16 {
        self.version
    }
    */

    pub fn nonce(&self) -> &str {
        &self.nonce
    }
    pub fn salt(&self) -> &str {
        &self.salt
    }
    pub fn cipher(&self) -> &str {
        &self.cipher
    }
}
