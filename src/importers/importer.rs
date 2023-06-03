use std::{error::Error, fs::read_to_string, path::PathBuf};

use serde::Deserialize;

use crate::otp::otp_element::OTPElement;

pub fn import_from_path<T>(path: PathBuf) -> Result<Vec<OTPElement>, Box<dyn Error>>
where
    T: for<'a> Deserialize<'a> + Into<OTPElement>,
{
    let json = read_to_string(path)?;
    import_from_string::<T>(&json)
}

/// Common logic for all the importers
pub fn import_from_string<T>(json: &str) -> Result<Vec<OTPElement>, Box<dyn Error>>
where
    T: for<'a> Deserialize<'a> + Into<OTPElement>,
{
    let deserialized: Vec<T> = serde_json::from_str(json)?;
    let mapped: Vec<OTPElement> = deserialized.into_iter().map(|e| e.into()).collect();
    Ok(mapped)
}
