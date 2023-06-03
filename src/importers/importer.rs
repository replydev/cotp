use std::{error::Error, fmt::Debug, fs::read_to_string, path::PathBuf};

use serde::Deserialize;

use crate::otp::otp_element::OTPElement;

/// Common flow for all the importers
pub fn import_from_path<T>(path: PathBuf) -> Result<Vec<OTPElement>, Box<dyn Error>>
where
    T: for<'a> Deserialize<'a> + TryInto<Vec<OTPElement>>,
    <T as TryInto<Vec<OTPElement>>>::Error: Debug,
{
    let json = read_to_string(path)?;
    let deserialized: T = serde_json::from_str(json.as_str()).map_err(|e| e.to_string())?;
    let mapped: Vec<OTPElement> = deserialized.try_into().map_err(|e| format!("{:?}", e))?;
    Ok(mapped)
}
