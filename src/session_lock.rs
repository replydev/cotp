//! Secret-free cross-process lock requests. Each dashboard watches the same marker.
use color_eyre::eyre::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn marker(path: &Path) -> Result<PathBuf> {
    let mut path = path.canonicalize()?.into_os_string();
    path.push(".lock-request");
    Ok(PathBuf::from(path))
}

pub fn request(path: &Path) -> Result<()> {
    let mut token = [0; 16];
    getrandom::fill(&mut token).map_err(|e| color_eyre::eyre::eyre!(e))?;
    crate::database_file::write(&marker(path)?, &token)?;
    Ok(())
}

pub struct Watcher {
    path: PathBuf,
    last: Option<Vec<u8>>,
}
impl Watcher {
    pub fn new(path: &Path) -> Result<Self> {
        let path = marker(path)?;
        let last = fs::read(&path).ok();
        Ok(Self { path, last })
    }
    pub fn requested(&mut self) -> bool {
        let current = fs::read(&self.path).ok();
        if current == self.last {
            return false;
        }
        let changed = current.is_some();
        self.last = current;
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_dashboards_receive_new_requests() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault");
        fs::write(&path, b"encrypted vault").unwrap();
        let mut first = Watcher::new(&path).unwrap();
        let mut second = Watcher::new(&path).unwrap();
        assert!(!first.requested());
        request(&path).unwrap();
        assert!(first.requested());
        assert!(second.requested());
        assert!(!first.requested());
        request(&path).unwrap();
        assert!(first.requested());
    }
}

#[derive(clap::Args)]
pub struct LockArgs {}
impl crate::arguments::SubcommandExecutor for LockArgs {
    fn run_command(
        self,
        database: crate::otp::otp_element::OTPDatabase,
    ) -> Result<crate::otp::otp_element::OTPDatabase> {
        Ok(database)
    }
}
