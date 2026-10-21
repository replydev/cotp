use std::{io::Write, path::PathBuf};

use serde::Serialize;

pub mod andotp;
pub mod freeotp_plus;
pub mod otp_uri;

pub fn do_export<T>(to_be_saved: &T, exported_path: PathBuf) -> Result<PathBuf, String>
where
    T: ?Sized + Serialize,
{
    let contents =
        zeroize::Zeroizing::new(serde_json::to_vec(to_be_saved).map_err(|e| e.to_string())?);
    if contents.as_slice() == b"[]" {
        return Err("No contents to export, skipping...".to_owned());
    }
    let parent = exported_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    file.write_all(&contents).map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist_noclobber(&exported_path).map_err(|e| {
        format!(
            "Cannot create export (existing files are never overwritten): {}",
            e.error
        )
    })?;
    Ok(exported_path)
}

pub mod aegis;
