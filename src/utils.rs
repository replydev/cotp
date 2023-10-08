use base64::{engine::general_purpose, Engine as _};
use copypasta_ext::prelude::*;
use copypasta_ext::x11_bin::ClipboardContext as BinClipboardContext;
use copypasta_ext::x11_fork::ClipboardContext as ForkClipboardContext;
use crossterm::style::Print;
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

fn in_ssh_shell() -> bool {
    return !env::var("SSH_CONNECTION")
        .unwrap_or_default()
        .trim()
        .is_empty();
}

pub fn copy_string_to_clipboard(content: String) -> Result<CopyType, ()> {
    if in_ssh_shell()
        && crossterm::execute!(
            io::stdout(),
            Print(format!(
                "\x1B]52;c;{}\x07",
                general_purpose::STANDARD.encode(&content)
            ))
        )
        .is_ok()
    {
        // We do not use copypasta_ext::osc52 module because we have enabled terminal raw mode, so we print with crossterm utilities
        // Check https://github.com/timvisee/rust-clipboard-ext/blob/371df19d2f961882a21c957f396d1e24548d1f28/src/osc52.rs#L92
        Ok(CopyType::OSC52)
    } else if BinClipboardContext::new()
        .and_then(|mut ctx| ctx.set_contents(content.clone()))
        .is_ok()
        || ForkClipboardContext::new()
            .and_then(|mut ctx| ctx.set_contents(content))
            .is_ok()
    {
        Ok(CopyType::Native)
    } else {
        Err(())
    }
}
