use crate::path::DATABASE_PATH;

pub fn init_app() -> Result<bool, ()> {
    let db_path = DATABASE_PATH.get().unwrap(); // Safe to unwrap because we initialize
    let db_dir = db_path.parent().unwrap();
    if !db_dir.exists() {
        if let Err(_e) = std::fs::create_dir_all(db_dir) {
            return Err(());
        }
        return Ok(true);
    }
    Ok(!db_path.exists())
}

pub fn password(message: &str, minimum_length: usize) -> String {
    loop {
        let password = rpassword::prompt_password(message).unwrap();
        if password.chars().count() < minimum_length {
            println!("Use a password with at least {minimum_length} characters.");
            continue;
        }
        return password;
    }
}

pub fn verified_password(message: &str, minimum_length: usize) -> String {
    loop {
        let password = password(message, minimum_length);
        let verify_password = rpassword::prompt_password("Confirm password: ").unwrap();
        if password != verify_password {
            println!("Passwords do not match. Try again.");
            continue;
        }
        return password;
    }
}
