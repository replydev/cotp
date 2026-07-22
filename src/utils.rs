use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use eyre::eyre;
use zeroize::Zeroize;

use crate::path::DATABASE_PATH;

pub fn init_app() -> eyre::Result<bool> {
    let db_path = DATABASE_PATH.get().unwrap(); // Safe to unwrap because we initialize

    // Decide whether this is a first run from the database file itself: relying on
    // the parent directory is wrong for bare relative paths (e.g. `-d db.cotp`),
    // whose parent is the empty path and never "exists", which previously caused an
    // existing database to be re-initialized and overwritten.
    if db_path.exists() {
        return Ok(false);
    }

    // First run: make sure the parent directory exists. An empty parent means the
    // current working directory.
    let db_dir = match db_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    if !db_dir.exists()
        && let Err(e) = std::fs::create_dir_all(db_dir)
    {
        return Err(eyre!(
            "Cannot create the database directory {}: {e}",
            db_dir.display()
        ));
    }
    Ok(true)
}

pub fn millis_before_next_step() -> u64 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    let in_ms = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1000000;
    in_ms % 30000
}

pub fn percentage() -> u16 {
    (millis_before_next_step() * 100 / 30000) as u16
}

/// Prompts for a password of at least `minimum_length` characters.
///
/// Reading the password can fail (e.g. stdin is not a TTY): the error is
/// propagated to the caller instead of exiting the process.
pub fn try_password(message: &str, minimum_length: usize) -> std::io::Result<String> {
    loop {
        let mut password = rpassword::prompt_password(message)?;
        if password.chars().count() < minimum_length {
            password.zeroize();
            println!("Please insert a password with at least {minimum_length} digits.");
            continue;
        }
        return Ok(password);
    }
}

/// Like [`try_password`], but asks the user to retype the password until both
/// entries match.
pub fn try_verified_password(message: &str, minimum_length: usize) -> std::io::Result<String> {
    loop {
        let mut password = try_password(message, minimum_length)?;
        let mut verify_password = match rpassword::prompt_password("Retype the same password: ") {
            Ok(verify_password) => verify_password,
            Err(e) => {
                password.zeroize();
                return Err(e);
            }
        };
        let matching = password == verify_password;
        verify_password.zeroize();
        if !matching {
            password.zeroize();
            println!("Passwords do not match");
            continue;
        }
        return Ok(password);
    }
}
