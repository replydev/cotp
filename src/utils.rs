use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;
use crossterm::style::Print;
#[cfg(not(debug_assertions))]
use dirs::home_dir;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, io};

pub enum CopyType {
    Native,
    OSC52,
}

pub fn get_db_path() -> PathBuf {
    match env::var("COTP_DB_PATH") {
        Ok(value) => PathBuf::from(value),
        Err(_e) => get_default_db_path(),
    }
}

// Pushing an absolute path to a PathBuf replaces the entire PathBuf: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push
pub fn get_default_db_path() -> PathBuf {
    let result: Option<PathBuf> = {
        #[cfg(not(debug_assertions))]
        {
            home_dir()
        }
        #[cfg(debug_assertions)]
        Some(PathBuf::from("."))
    };
    match result {
        Some(home) => home,
        None => {
            let current_dir = PathBuf::from(".");
            if let Some(str_dir) = current_dir.to_str() {
                eprintln!("Cannot get home folder, using: {}", str_dir);
            } else {
                eprintln!("Cannot get home folder, using");
            }
            current_dir
        }
    }
    .join(".cotp/db.cotp")
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
            println!(
                "Please insert a password with at least {} digits.",
                minimum_length
            );
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

fn in_ssh_shell() -> bool {
    return !env::var("SSH_CONNECTION")
        .unwrap_or_default()
        .trim()
        .is_empty();
}

pub fn copy_string_to_clipboard(content: String) -> Result<CopyType, ()> {
    if in_ssh_shell() {
        // We do not use copypasta_ext::osc52 module because we have enabled terminal raw mode, so we print with crossterm utilities
        // Check https://github.com/timvisee/rust-clipboard-ext/blob/371df19d2f961882a21c957f396d1e24548d1f28/src/osc52.rs#L92
        return match crossterm::execute!(
            io::stdout(),
            Print(format!("\x1B]52;c;{}\x07", base64::encode(content)))
        ) {
            Ok(_) => Ok(CopyType::OSC52),
            Err(_) => Err(()),
        };
    } else if let Ok(mut ctx) = ClipboardContext::new() {
        return if ctx.set_contents(content).is_ok() {
            Ok(CopyType::Native)
        } else {
            Err(())
        };
    }
    Err(())
}
