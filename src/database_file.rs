use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

pub fn write(path: &Path, content: &[u8]) -> io::Result<()> {
    replace_with(path, |file| {
        file.write_all(content)?;
        file.sync_all()
    })
}

fn replace_with(
    path: &Path,
    write_and_sync: impl FnOnce(&mut File) -> io::Result<()>,
) -> io::Result<()> {
    // Follow existing symlinks, as opening the database directly did before.
    let path = match path.canonicalize() {
        Ok(path) => path,
        Err(e) if e.kind() == io::ErrorKind::NotFound => path.to_path_buf(),
        Err(e) => return Err(e),
    };
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    // Keeping the temporary file beside the database allows an atomic replacement.
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    match fs::metadata(&path) {
        Ok(metadata) => temporary
            .as_file()
            .set_permissions(metadata.permissions())?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    write_and_sync(temporary.as_file_mut())?;
    temporary.persist(&path).map_err(|e| e.error)?;
    // Persist the directory entry as well on platforms that support directory sync.
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_existing_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db.cotp");
        fs::write(&path, b"old database with a longer payload").unwrap();

        write(&path, b"new database").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"new database");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn creates_new_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db.cotp");

        write(&path, b"new database").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"new database");
    }

    #[test]
    fn failed_write_preserves_database_and_removes_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db.cotp");
        fs::write(&path, b"original database").unwrap();

        let error = replace_with(&path, |file| {
            file.write_all(b"partial replacement")?;
            Err(io::Error::from(io::ErrorKind::StorageFull))
        })
        .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::StorageFull);
        assert_eq!(fs::read(&path).unwrap(), b"original database");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn failed_persist_removes_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db.cotp");
        fs::create_dir(&path).unwrap();

        assert!(write(&path, b"new database").is_err());

        assert!(path.is_dir());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn preserves_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db.cotp");
        fs::write(&path, b"old database").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();

        write(&path, b"new database").unwrap();

        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }

    #[cfg(unix)]
    #[test]
    fn updates_symlink_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db.cotp");
        let link = dir.path().join("link.cotp");
        fs::write(&path, b"old database").unwrap();
        std::os::unix::fs::symlink(&path, &link).unwrap();

        write(&link, b"new database").unwrap();

        assert!(fs::symlink_metadata(&link).unwrap().is_symlink());
        assert_eq!(fs::read(&path).unwrap(), b"new database");
    }
}
