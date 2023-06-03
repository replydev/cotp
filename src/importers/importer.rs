use std::{error::Error, fs::read_to_string, path::PathBuf};

use serde::Deserialize;

use crate::otp::otp_element::OTPElement;

enum ImportError {}

pub struct Importer {
    path: PathBuf,
}

impl Importer {
    pub fn new(path: PathBuf) -> Importer {
        Importer { path }
    }

    /// Common logic for all the importers
    pub fn import<T>(&self) -> Result<Vec<OTPElement>, Box<dyn Error>>
    where
        T: for<'a> Deserialize<'a> + Into<OTPElement>,
    {
        let json = read_to_string(self.path)?;
        let deserialized: Vec<T> = serde_json::from_str(json.as_str())?;
        let mapped: Vec<OTPElement> = deserialized.into_iter().map(|e| e.into()).collect();
        Ok(mapped)
    }
}
