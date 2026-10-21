//! Backups are authenticated encrypted vault files, never plaintext exports.
use crate::{
    crypto::{cryptography::decrypt_string, encrypted_database::EncryptedDatabase},
    otp::{
        migrations::migrate,
        otp_element::{OTPDatabase, OTPElement},
    },
    persistence::VaultLock,
};
use clap::{Args, Subcommand};
use color_eyre::eyre::{Result, eyre};
use std::{
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroizing;

#[derive(Args)]
pub struct BackupArgs {
    #[command(subcommand)]
    pub action: BackupAction,
}

#[derive(Subcommand)]
pub enum BackupAction {
    /// Copy the encrypted vault into a new private backup file
    Create {
        directory: PathBuf,
        /// Keep only this many cotp backups in the directory (at least one)
        #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
        keep: Option<u16>,
    },
    /// List encrypted backup files without unlocking the vault
    List { directory: PathBuf },
    /// Authenticate and validate a backup using its original password
    Verify { file: PathBuf },
    /// Restore an authenticated backup; its original password remains in effect
    Restore {
        file: PathBuf,
        /// Replace an existing vault (may roll back HOTP counters)
        #[arg(long)]
        force: bool,
    },
}

impl crate::arguments::SubcommandExecutor for BackupArgs {
    fn run_command(self, _: OTPDatabase) -> Result<OTPDatabase> {
        Err(eyre!("Backup commands must run before opening the vault"))
    }
}

pub fn run(args: &BackupArgs, vault: &Path, password_stdin: bool) -> Result<()> {
    match &args.action {
        BackupAction::Create { directory, keep } => {
            let path = create(vault, directory)?;
            if let Some(keep) = keep {
                prune(directory, usize::from(*keep))?;
            }
            println!("{}", path.display());
        }
        BackupAction::List { directory } => {
            for path in list(directory)? {
                println!("{}", path.display());
            }
        }
        BackupAction::Verify { file } => {
            let password = password(password_stdin)?;
            let bytes = fs::read(file)?;
            let database = validate(&bytes, &password)?;
            println!(
                "Verified encrypted backup: {} accounts",
                database.elements_ref().len()
            );
        }
        BackupAction::Restore { file, force } => {
            let password = password(password_stdin)?;
            restore(file, vault, &password, *force)?;
            eprintln!(
                "Encrypted backup restored. Use the backup's original password to unlock it."
            );
        }
    }
    Ok(())
}

fn password(from_stdin: bool) -> Result<Zeroizing<String>> {
    let password = if from_stdin {
        io::stdin()
            .lock()
            .lines()
            .next()
            .ok_or_else(|| eyre!("Missing backup password on stdin"))??
    } else {
        rpassword::prompt_password("Backup password: ")?
    };
    Ok(Zeroizing::new(password))
}

fn validate(bytes: &[u8], password: &str) -> Result<OTPDatabase> {
    let text = std::str::from_utf8(bytes)?;
    let (plaintext, key, _salt) = decrypt_string(text, password)?;
    let plaintext = Zeroizing::new(plaintext);
    let _key = Zeroizing::new(key);
    let mut database: OTPDatabase = serde_json::from_str(&plaintext)
        .or_else(|_| serde_json::from_str::<Vec<OTPElement>>(&plaintext).map(Into::into))?;
    if database.version > crate::otp::otp_element::CURRENT_DATABASE_VERSION {
        return Err(eyre!("Backup uses a newer unsupported vault format"));
    }
    migrate(&mut database)?;
    for element in database.elements_ref() {
        element
            .get_otp_code()
            .map_err(|_| eyre!("Backup contains an invalid OTP account"))?;
    }
    Ok(database)
}

fn private_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(path)?;
    }
    Ok(())
}

fn create(vault: &Path, directory: &Path) -> Result<PathBuf> {
    let bytes = {
        let _lock = VaultLock::acquire(vault)?;
        fs::read(vault)?
    };
    // Reject accidentally pointing backup at a plaintext export.
    let _: EncryptedDatabase = serde_json::from_slice(&bytes)?;
    private_directory(directory)?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = directory.join(format!("cotp-{timestamp:030}.cotp-backup"));
    // No-clobber persist prevents replacing an existing backup on a timestamp collision.
    let mut temporary = tempfile::Builder::new()
        .prefix(".cotp-backup-")
        .tempfile_in(directory)?;
    use std::io::Write;
    temporary.write_all(&bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist_noclobber(&path).map_err(|e| e.error)?;
    #[cfg(unix)]
    fs::File::open(directory)?.sync_all()?;
    Ok(path)
}

fn list(directory: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = vec![];
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("cotp-") && name.ends_with(".cotp-backup"))
        {
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok(paths)
}

fn prune(directory: &Path, keep: usize) -> Result<()> {
    let paths = list(directory)?;
    let remove = paths.len().saturating_sub(keep);
    for path in paths.into_iter().take(remove) {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn restore(file: &Path, vault: &Path, password: &str, force: bool) -> Result<()> {
    let bytes = fs::read(file)?;
    validate(&bytes, password)?;
    if let Some(parent) = vault.parent().filter(|p| !p.as_os_str().is_empty()) {
        private_directory(parent)?;
    }
    let _lock = VaultLock::acquire(vault)?;
    if vault.exists() && !force {
        return Err(eyre!(
            "Vault already exists. Use --force to replace it; restoring may roll back HOTP counters."
        ));
    }
    crate::database_file::write(vault, &bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::cryptography::{argon_derive_key, encrypt_string_with_key};
    fn encrypted() -> Vec<u8> {
        let salt = [42; 16];
        let key = argon_derive_key(b"backup password", &salt).unwrap();
        serde_json::to_vec(
            &encrypt_string_with_key(
                &serde_json::to_string(&OTPDatabase::default()).unwrap(),
                &key,
                &salt,
            )
            .unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn encrypted_round_trip_and_failed_restore_preserves_destination() {
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault");
        let original = encrypted();
        fs::write(&vault, &original).unwrap();
        let backup = create(&vault, &dir.path().join("backups")).unwrap();
        assert_eq!(fs::read(&backup).unwrap(), original);
        assert!(validate(&original, "backup password").is_ok());
        fs::write(&vault, b"existing").unwrap();
        assert!(restore(&backup, &vault, "wrong password", true).is_err());
        assert_eq!(fs::read(&vault).unwrap(), b"existing");
        assert!(restore(&backup, &vault, "backup password", false).is_err());
        assert_eq!(fs::read(&vault).unwrap(), b"existing");
        restore(&backup, &vault, "backup password", true).unwrap();
        assert_eq!(fs::read(&vault).unwrap(), original);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&backup).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
    #[test]
    fn retention_only_removes_matching_backup_files() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "cotp-0001.cotp-backup",
            "cotp-0002.cotp-backup",
            "cotp-0003.cotp-backup",
            "keep.json",
        ] {
            fs::write(dir.path().join(name), b"data").unwrap();
        }
        prune(dir.path(), 2).unwrap();
        assert!(!dir.path().join("cotp-0001.cotp-backup").exists());
        assert!(dir.path().join("keep.json").exists());
        assert_eq!(list(dir.path()).unwrap().len(), 2);
    }
}
