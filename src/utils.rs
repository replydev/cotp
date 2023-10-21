use crate::path::get_db_path;
use std::time::{SystemTime, UNIX_EPOCH};

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
