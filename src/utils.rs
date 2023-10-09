use dirs::home_dir;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, io};

pub fn get_db_path() -> PathBuf {
    match env::var("COTP_DB_PATH") {
        Ok(value) => PathBuf::from(value),
        Err(_e) => get_default_db_path(),
    }
}

pub fn is_portable_mode() -> bool {
    PathBuf::from("db.cotp").exists()
}

// Pushing an absolute path to a PathBuf replaces the entire PathBuf: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push
pub fn get_default_db_path() -> PathBuf {
    let db_from_current_dir = PathBuf::from("./db.cotp");

    // If db.cotp is present in the current directory or we are using a debug artifact, do not use the one in home dir
    // First condition is optimized away in release mode
    if cfg!(debug_assertions) || is_portable_mode() {
        return db_from_current_dir;
    }

    // Take from homedir, otherwise fallback to portable mode
    home_dir()
        .map(|path| path.join(".cotp/db.cotp"))
        .unwrap_or(db_from_current_dir)
}

pub fn init_app() -> Result<bool, ()> {
    let db_path = get_db_path();
    let db_dir = db_path.parent().unwrap();
    if !db_dir.exists() {
        if let Err(_e) = std::fs::create_dir_all(db_dir) {
            return Err(());
        }
        return Ok(true);
    }
    Ok(!db_path.exists())
}

pub fn delete_db() -> io::Result<()> {
    std::fs::remove_file(get_db_path())
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

pub fn password(message: &str, minimum_length: usize) -> String {
    loop {
        let password = rpassword::prompt_password(message).unwrap();
        if password.chars().count() < minimum_length {
            println!("Please insert a password with at least {minimum_length} digits.");
            continue;
        }
        return password;
    }
}

pub fn verified_password(message: &str, minimum_length: usize) -> String {
    loop {
        let password = password(message, minimum_length);
        let verify_password = rpassword::prompt_password("Retype the same password: ").unwrap();
        if password != verify_password {
            println!("Passwords do not match");
            continue;
        }
        return password;
    }
}
