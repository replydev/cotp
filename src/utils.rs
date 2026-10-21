use crate::path::DATABASE_PATH;
use color_eyre::eyre::eyre;
use std::io::{self, BufRead, IsTerminal};
use std::sync::atomic::{AtomicBool, Ordering};
use zeroize::Zeroize;

static NO_INPUT: AtomicBool = AtomicBool::new(false);

pub fn set_no_input(value: bool) {
    NO_INPUT.store(value, Ordering::Relaxed);
}

pub fn no_input() -> bool {
    NO_INPUT.load(Ordering::Relaxed)
}

pub fn ensure_interactive() -> color_eyre::Result<()> {
    if no_input() || !io::stdin().is_terminal() {
        return Err(eyre!(
            "Interactive input is disabled; provide explicit input options (see --help)"
        ));
    }
    Ok(())
}

pub fn prompt_password(message: &str) -> color_eyre::Result<String> {
    ensure_interactive()?;
    Ok(rpassword::prompt_password(message)?)
}

pub fn stdin_line() -> color_eyre::Result<String> {
    read_line(&mut io::stdin().lock())
}

pub fn read_line(reader: &mut impl BufRead) -> color_eyre::Result<String> {
    let mut value = String::new();
    if reader.read_line(&mut value)? == 0 {
        return Err(eyre!("Input ended before a value was supplied"));
    }
    if value.ends_with('\n') {
        value.pop();
        if value.ends_with('\r') {
            value.pop();
        }
    }
    Ok(value)
}

pub fn password_from_fd(fd: u32) -> color_eyre::Result<String> {
    if fd == 0 {
        return stdin_line();
    }
    #[cfg(unix)]
    {
        read_line(&mut io::BufReader::new(std::fs::File::open(format!(
            "/dev/fd/{fd}"
        ))?))
    }
    #[cfg(not(unix))]
    {
        let _ = fd;
        Err(eyre!(
            "Password file descriptors are supported on Unix; use --password-stdin on this platform"
        ))
    }
}

pub fn init_app() -> Result<bool, ()> {
    let db_path = DATABASE_PATH.get().unwrap();
    let db_dir = db_path.parent().unwrap();
    if !db_dir.as_os_str().is_empty() && !db_dir.exists() {
        std::fs::create_dir_all(db_dir).map_err(|_| ())?;
    }
    Ok(!db_path.exists())
}

pub fn password(message: &str, minimum_length: usize) -> color_eyre::Result<String> {
    loop {
        let mut password = prompt_password(message)?;
        if password.chars().count() < minimum_length {
            password.zeroize();
            eprintln!("Please insert a password with at least {minimum_length} characters.");
            continue;
        }
        return Ok(password);
    }
}

pub fn verified_password(message: &str, minimum_length: usize) -> color_eyre::Result<String> {
    loop {
        let mut password = password(message, minimum_length)?;
        let mut verify_password = prompt_password("Retype the same password: ")?;
        let matches = password == verify_password;
        verify_password.zeroize();
        if !matches {
            password.zeroize();
            eprintln!("Passwords do not match");
            continue;
        }
        return Ok(password);
    }
}
