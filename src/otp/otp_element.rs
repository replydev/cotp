use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OTPElement {
    secret: String,
    issuer: String,
    label: String,
    digits: u64,
    #[serde(rename = "type")]
    type_: String,
    algorithm: String,
    thumbnail: String,
    last_used: u64,
    used_frequency: u64,
    period: u64,
    counter: Option<u64>,
    tags: Vec<String>,
}

impl OTPElement {
    pub fn new(secret: String, issuer: String, label: String, digits: u64, type_: String, algorithm: String, thumbnail: String, last_used: u64, used_frequency: u64, period: u64, counter: u64, tags: Vec<String>,
    ) -> OTPElement {
        OTPElement {
            secret,
            issuer,
            label,
            digits,
            type_,
            algorithm,
            thumbnail,
            last_used,
            used_frequency,
            period,
            counter: Some(counter),
            tags,
        }
    }
    pub fn secret(&self) -> String {
        self.secret.to_string().replace("=", "")
    }
    pub fn label(&self) -> String {
        self.label.to_string()
    }
    pub fn issuer(&self) -> String {
        self.issuer.to_string()
    }
    pub fn digits(&self) -> u64 {
        self.digits
    }
    pub fn algorithm(&self) -> String {
        self.algorithm.to_string()
    }
    pub fn type_(&self) -> String { 
        self.type_.to_string() 
    }
    pub fn counter(&self) -> Option<u64> { 
        self.counter
    }

    pub fn set_secret(&mut self, secret: String) {
        self.secret = secret;
    }
    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }
    pub fn set_issuer(&mut self, issuer: String) {
        self.issuer = issuer;
    }
    pub fn set_algorithm(&mut self, algorithm: String) {
        self.algorithm = algorithm;
    }
    pub fn set_digits(&mut self, digits: u64) {
        self.digits = digits;
    }
    pub fn set_counter(&mut self, counter: Option<u64>) { 
        self.counter = counter; 
    }
}