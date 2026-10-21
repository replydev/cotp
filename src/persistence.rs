//! Optimistic vault transactions protected by an advisory cross-process lock.
use crate::otp::{otp_element::OTPDatabase, otp_type::OTPType};
use color_eyre::eyre::{Result, eyre};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    sync::Mutex,
};
use zeroize::Zeroizing;

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

pub(crate) struct VaultLock {
    _file: File,
}
impl VaultLock {
    pub(crate) fn acquire(path: &Path) -> Result<Self> {
        let path = normalized_path(path)?;
        let mut lock_name = path.as_os_str().to_os_string();
        lock_name.push(".lock");
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(PathBuf::from(lock_name))?;
        file.try_lock()
            .map_err(|e| eyre!("Vault is busy; another process is writing it: {e}"))?;
        Ok(Self { _file: file })
    }
}

fn normalized_path(path: &Path) -> Result<PathBuf> {
    match path.canonicalize() {
        Ok(path) => Ok(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            Ok(parent.canonicalize()?.join(
                path.file_name()
                    .ok_or_else(|| eyre!("Invalid vault path"))?,
            ))
        }
        Err(e) => Err(e.into()),
    }
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

struct Session {
    path: PathBuf,
    snapshot: Option<Vec<u8>>,
    key: Option<Zeroizing<Vec<u8>>>,
    salt: Vec<u8>,
}
impl Session {
    fn open(path: &Path) -> Result<Self> {
        let path = normalized_path(path)?;
        let _lock = VaultLock::acquire(&path)?;
        let snapshot = read_optional(&path)?;
        Ok(Self {
            path,
            snapshot,
            key: None,
            salt: vec![],
        })
    }
    fn write(&mut self, content: &[u8]) -> Result<()> {
        let _lock = VaultLock::acquire(&self.path)?;
        if read_optional(&self.path)? != self.snapshot {
            return Err(eyre!(
                "Vault changed in another process. Reopen the vault and retry; your changes were not written."
            ));
        }
        crate::database_file::write(&self.path, content)?;
        self.snapshot = Some(content.to_vec());
        Ok(())
    }
}

pub fn initialize(path: &Path) -> Result<()> {
    *SESSION
        .lock()
        .map_err(|_| eyre!("Vault session unavailable"))? = Some(Session::open(path)?);
    Ok(())
}

pub fn unlock(key: &[u8], salt: &[u8]) -> Result<()> {
    let mut session = SESSION
        .lock()
        .map_err(|_| eyre!("Vault session unavailable"))?;
    let session = session
        .as_mut()
        .ok_or_else(|| eyre!("Vault session not initialized"))?;
    session.key = Some(Zeroizing::new(key.to_vec()));
    session.salt = salt.to_vec();
    Ok(())
}

pub fn lock() {
    if let Ok(mut session) = SESSION.lock()
        && let Some(session) = session.as_mut()
    {
        session.key = None;
        session.salt.clear();
    }
}

pub fn write_database(path: &Path, content: &[u8]) -> Result<()> {
    let mut session = SESSION
        .lock()
        .map_err(|_| eyre!("Vault session unavailable"))?;
    let session = session
        .as_mut()
        .ok_or_else(|| eyre!("Vault session not initialized"))?;
    if normalized_path(path)? != session.path {
        return Err(eyre!("Vault path changed during session"));
    }
    session.write(content)
}

pub fn persist(database: &mut OTPDatabase) -> Result<()> {
    let (key, salt) = {
        let session = SESSION
            .lock()
            .map_err(|_| eyre!("Vault session unavailable"))?;
        let session = session
            .as_ref()
            .ok_or_else(|| eyre!("Vault session not initialized"))?;
        let key = session
            .key
            .as_ref()
            .ok_or_else(|| eyre!("Vault is locked"))?;
        (Zeroizing::new(key.to_vec()), session.salt.clone())
    };
    database.save(&key, &salt)
}

/// Persist the next counter before handing a consumable HOTP code to a caller.
/// Pending user edits remain pending and are never silently committed by copying.
pub fn consume_hotp(database: &mut OTPDatabase, index: usize) -> Result<String> {
    consume_with(database, index, persist)
}

fn consume_with(
    database: &mut OTPDatabase,
    index: usize,
    save: impl FnOnce(&mut OTPDatabase) -> Result<()>,
) -> Result<String> {
    if database.is_modified() {
        return Err(eyre!("Save pending changes before consuming HOTP"));
    }
    let element = database
        .mut_element(index)
        .ok_or_else(|| eyre!("Account not found"))?;
    if element.type_ != OTPType::Hotp {
        return Err(eyre!("Selected account is not HOTP"));
    }
    let counter = element
        .counter
        .ok_or_else(|| eyre!("Missing HOTP counter"))?;
    let next = counter
        .checked_add(1)
        .ok_or_else(|| eyre!("HOTP counter exhausted"))?;
    let code = Zeroizing::new(element.get_otp_code()?);
    element.counter = Some(next);
    database.mark_modified();
    if let Err(error) = save(database) {
        database.mut_element(index).unwrap().counter = Some(counter);
        database.needs_modification = false;
        return Err(error);
    }
    Ok(code.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::otp_element::OTPElementBuilder;

    #[test]
    fn contention_and_stale_writes_preserve_winner() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault");
        fs::write(&path, b"initial").unwrap();
        let mut first = Session::open(&path).unwrap();
        let mut second = Session::open(&path).unwrap();
        let lock = VaultLock::acquire(&path).unwrap();
        assert!(VaultLock::acquire(&path).is_err());
        assert!(first.write(b"blocked").is_err());
        drop(lock);
        first.write(b"winner").unwrap();
        assert!(second.write(b"stale").is_err());
        assert_eq!(fs::read(&path).unwrap(), b"winner");
        first.write(b"second commit").unwrap();
    }

    #[test]
    fn lock_child() {
        if let Some(path) = std::env::var_os("COTP_LOCK_TEST_PATH") {
            assert!(VaultLock::acquire(Path::new(&path)).is_err());
        }
    }

    #[test]
    fn lock_excludes_another_process() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault");
        let _lock = VaultLock::acquire(&path).unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "persistence::tests::lock_child"])
            .env("COTP_LOCK_TEST_PATH", path)
            .status()
            .unwrap();
        assert!(status.success());
    }

    fn hotp_database() -> OTPDatabase {
        let element = OTPElementBuilder::default()
            .secret("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")
            .issuer("test")
            .label("test")
            .type_(OTPType::Hotp)
            .counter(0)
            .build()
            .unwrap();
        OTPDatabase {
            elements: vec![element],
            ..Default::default()
        }
    }

    #[test]
    fn hotp_failure_rolls_back_and_success_commits_before_release() {
        let mut db = hotp_database();
        assert!(consume_with(&mut db, 0, |_| Err(eyre!("Disk full"))).is_err());
        assert_eq!(db.elements[0].counter, Some(0));
        assert!(!db.is_modified());
        let result = consume_with(&mut db, 0, |db| {
            assert_eq!(db.elements[0].counter, Some(1));
            db.needs_modification = false;
            Ok(())
        })
        .unwrap();
        assert_eq!(result, "755224");
        assert_eq!(db.elements[0].counter, Some(1));
    }

    #[test]
    fn pending_changes_and_exhausted_counter_are_not_consumed() {
        let mut db = hotp_database();
        db.mark_modified();
        assert!(consume_with(&mut db, 0, |_| panic!("must not save")).is_err());
        assert!(db.is_modified());
        assert_eq!(db.elements[0].counter, Some(0));
        db.needs_modification = false;
        db.elements[0].counter = Some(u64::MAX);
        assert!(consume_with(&mut db, 0, |_| panic!("must not save")).is_err());
    }
}
