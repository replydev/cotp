use std::{fmt::Debug, fs::read_to_string, path::PathBuf};

use color_eyre::eyre::{eyre, Result};
use serde::Deserialize;

use crate::otp::otp_element::OTPElement;

/// Common flow for all the importers
pub fn import_from_path<T>(path: PathBuf) -> Result<Vec<OTPElement>>
where
    T: for<'a> Deserialize<'a> + TryInto<Vec<OTPElement>>,
    <T as TryInto<Vec<OTPElement>>>::Error: Debug,
{
    let json = read_to_string(path)?;
    let deserialized: T = serde_json::from_str(json.as_str()).map_err(|e| {
        eyre!(
            "Invalid JSON import format.
            Please check the file you are trying to import. For further information please check these guidelines:
            https://github.com/replydev/cotp?tab=readme-ov-file#migration-from-other-apps
            
            Specific error: {:?}",
            e
        )
    })?;
    let mapped: Vec<OTPElement> = deserialized.try_into().map_err(|e| eyre!("{:?}", e))?;
    Ok(mapped)
}
