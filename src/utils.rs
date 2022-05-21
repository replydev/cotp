use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use copypasta_ext::osc52::Osc52ClipboardContext;
use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;
#[cfg(not(debug_assertions))]
use dirs::home_dir;

use crate::otp::otp_element::OTPElement;

pub enum CopyType {
    NATIVE,
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

pub fn create_db_if_needed() -> Result<bool, ()> {
    let db_path = get_db_path();
    let db_dir = db_path.parent().unwrap();
    if !db_dir.exists() {
        if let Err(_e) = std::fs::create_dir(db_dir) {
            return Err(());
        }
    }
    if !db_path.exists() {
        return match File::create(db_path) {
            Ok(_f) => Ok(true),
            Err(_e) => Err(()),
        };
    }
    Ok(false)
}

pub fn delete_db() -> std::io::Result<()> {
    std::fs::remove_file(get_db_path())
}

pub fn write_to_file(content: &str, file: &mut File) -> Result<(), std::io::Error> {
    file.write_all(content.as_bytes())?;
    file.sync_all()
}

pub fn check_elements(id: usize, elements: &[OTPElement]) -> Result<(), String> {
    if elements.is_empty() {
        return Err(String::from(
            "there are no elements in your database. Type \"cotp -h\" to get help.",
        ));
    }

    if id >= elements.len() {
        return Err(format!("{} is a bad index", id + 1));
    }

    Ok(())
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

pub fn prompt_for_passwords(message: &str, minimum_password_length: usize, verify: bool) -> String {
    let mut password;
    loop {
        password = rpassword::prompt_password(message).unwrap();
        if verify {
            let verify_password = rpassword::prompt_password("Retype the same password: ").unwrap();
            if password != verify_password {
                println!("Passwords do not match");
                continue;
            }
        }
        if password.chars().count() >= minimum_password_length {
            break;
        }
        println!(
            "Please insert a password with at least {} digits.",
            minimum_password_length
        );
    }
    password
}

fn in_ssh_shell() -> bool {
    return !env::var("SSH_CONNECTION")
        .unwrap_or_default()
        .trim()
        .is_empty();
}

pub fn copy_string_to_clipboard(content: String) -> Result<CopyType, ()> {
    if in_ssh_shell() {
        if let Ok(mut ctx) = Osc52ClipboardContext::new() {
            return if ctx.set_contents(content).is_ok() {
                Ok(CopyType::OSC52)
            } else {
                Err(())
            };
        }
    } else {
        if let Ok(mut ctx) = ClipboardContext::new() {
            return if ctx.set_contents(content).is_ok() {
                Ok(CopyType::NATIVE)
            } else {
                Err(())
            };
        }
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use super::create_db_if_needed;

    #[test]
    fn test_db_creation() {
        assert!(create_db_if_needed().is_ok());
    }
}
