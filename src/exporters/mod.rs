use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::{Path, PathBuf},
};

use eyre::eyre;
use serde::Serialize;
use zeroize::Zeroize;

pub mod andotp;
pub mod freeotp_plus;
pub mod otp_uri;

pub fn do_export<T>(to_be_saved: &T, exported_path: PathBuf) -> eyre::Result<PathBuf>
where
    T: ?Sized + Serialize,
{
    let mut contents = match serde_json::to_string(to_be_saved) {
        Ok(contents) => contents,
        Err(e) => return Err(eyre!("Failed to serialize the export: {e}")),
    };
    if contents == "[]" {
        contents.zeroize();
        return Err(eyre!("No contents to export, skipping..."));
    }
    let write_result = write_secret_file(&exported_path, contents.as_bytes());
    contents.zeroize();
    match write_result {
        Ok(()) => {
            eprintln!(
                "Warning: the exported file contains your OTP secrets in PLAIN TEXT. Keep it safe and delete it as soon as it is no longer needed."
            );
            Ok(exported_path)
        }
        Err(e) => Err(eyre!(
            "Cannot export to file {}: {e}",
            exported_path.display()
        )),
    }
}

/// Writes the plain text export, creating the file with owner-only permissions
/// (0600) on Unix so other local users cannot read the exported secrets.
fn write_secret_file(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(contents)
}

#[cfg(test)]
mod tests {
    use super::do_export;

    #[test]
    fn export_error_is_propagated_instead_of_panicking() {
        let result = do_export(
            &vec!["some content"],
            std::path::PathBuf::from("/nonexistent-dir/never/created/export.json"),
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .starts_with("Cannot export to file")
        );
    }

    #[test]
    fn empty_export_is_skipped() {
        let empty: Vec<String> = vec![];
        let result = do_export(&empty, std::path::PathBuf::from("unused.json"));
        assert_eq!(
            result.unwrap_err().to_string(),
            "No contents to export, skipping..."
        );
    }

    #[cfg(unix)]
    #[test]
    fn exported_file_is_created_with_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("cotp-export-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("export.json");
        let exported = do_export(&vec!["secret"], path.clone()).unwrap();
        let mode = std::fs::metadata(&exported).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
