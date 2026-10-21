use base64::{Engine as _, engine::general_purpose};
use color_eyre::eyre::eyre;
#[cfg(feature = "desktop-clipboard")]
use copypasta_ext::prelude::*;
#[cfg(all(feature = "desktop-clipboard", target_os = "linux"))]
use copypasta_ext::wayland_bin::WaylandBinClipboardContext;
#[cfg(feature = "desktop-clipboard")]
use copypasta_ext::{
    x11_bin::ClipboardContext as BinClipboardContext,
    x11_fork::ClipboardContext as ForkClipboardContext,
};
use crossterm::style::Print;
use std::{
    env,
    io::{self, Read, Write},
    process::{Command, Stdio},
    time::Duration,
};
use zeroize::Zeroizing;

pub enum CopyType {
    Native,
    OSC52,
}

/// Native expiration compares the clipboard with our copy before clearing it.
/// OSC52 has no portable read-back, so it is never cleared automatically.
pub fn copy_string_to_clipboard(content: &str) -> color_eyre::Result<CopyType> {
    if env_var_set("SSH_CONNECTION") {
        crossterm::execute!(
            io::stderr(),
            Print(format!(
                "\x1B]52;c;{}\x07",
                general_purpose::STANDARD.encode(content)
            ))
        )?;
        return Ok(CopyType::OSC52);
    }
    for backend in ["wayland", "bin", "fork"] {
        if native_set(backend, content) {
            let seconds = crate::config::settings().clipboard_seconds;
            if seconds > 0 {
                schedule_expiration(backend, content, seconds)?;
            }
            return Ok(CopyType::Native);
        }
    }
    Err(eyre!("Cannot detect clipboard implementation"))
}

fn schedule_expiration(backend: &str, content: &str, seconds: u64) -> color_eyre::Result<()> {
    // The short-lived CLI cannot own the timer. Pass the sensitive content through
    // an anonymous pipe; argv and environment contain only public timer metadata.
    let mut child = Command::new(env::current_exe()?)
        .args(["--internal-clipboard-expiry", backend, &seconds.to_string()])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(mut input) = child.stdin.take() {
        input.write_all(content.as_bytes())?;
    }
    // Reap the helper in long-running dashboards; the OS reaps it after CLI exit.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Handle the private helper before parsing CLI arguments or unlocking a vault.
pub fn run_expiry_helper() -> color_eyre::Result<bool> {
    let args: Vec<_> = env::args().collect();
    if args.get(1).map(String::as_str) == Some("--internal-clipboard-write") {
        let mut content = Zeroizing::new(String::new());
        io::stdin().take(1_048_577).read_to_string(&mut content)?;
        if content.len() > 1_048_576 || !fork_set(&content) {
            return Err(eyre!("Cannot write native clipboard"));
        }
        return Ok(true);
    }
    if args.get(1).map(String::as_str) != Some("--internal-clipboard-expiry") {
        return Ok(false);
    }
    if args.len() != 4 {
        return Err(eyre!("Invalid clipboard helper arguments"));
    }
    let seconds = args[3].parse::<u64>()?;
    let mut content = Zeroizing::new(String::new());
    io::stdin().take(1_048_577).read_to_string(&mut content)?;
    if content.len() > 1_048_576 {
        return Err(eyre!("Clipboard content too large"));
    }
    std::thread::sleep(Duration::from_secs(seconds));
    clear_owned(
        &content,
        || native_get(&args[2]),
        |value| native_set(&args[2], value),
    );
    Ok(true)
}

fn clear_owned(
    expected: &str,
    read: impl FnOnce() -> Option<String>,
    clear: impl FnOnce(&str) -> bool,
) -> bool {
    if let Some(current) = read() {
        let current = Zeroizing::new(current);
        if current.as_str() == expected {
            return clear("");
        }
    }
    false
}

#[cfg(feature = "desktop-clipboard")]
fn native_set(backend: &str, content: &str) -> bool {
    match backend {
        #[cfg(target_os = "linux")]
        "wayland" if env_var_set("WAYLAND_DISPLAY") => WaylandBinClipboardContext::new()
            .and_then(|mut ctx| ctx.set_contents(content.to_owned()))
            .is_ok(),
        "bin" => BinClipboardContext::new()
            .and_then(|mut ctx| ctx.set_contents(content.to_owned()))
            .is_ok(),
        // X11's provider forks. Isolate it first, so no fork can inherit the
        // dashboard's decrypted vault, editor buffers or encryption key.
        "fork" => isolated_fork_set(content),
        _ => false,
    }
}
#[cfg(feature = "desktop-clipboard")]
fn native_get(backend: &str) -> Option<String> {
    match backend {
        #[cfg(target_os = "linux")]
        "wayland" => WaylandBinClipboardContext::new()
            .and_then(|mut ctx| ctx.get_contents())
            .ok(),
        "bin" => BinClipboardContext::new()
            .and_then(|mut ctx| ctx.get_contents())
            .ok(),
        "fork" => ForkClipboardContext::new()
            .and_then(|mut ctx| ctx.get_contents())
            .ok(),
        _ => None,
    }
}
#[cfg(not(feature = "desktop-clipboard"))]
fn native_set(_backend: &str, _content: &str) -> bool {
    false
}
#[cfg(not(feature = "desktop-clipboard"))]
fn native_get(_backend: &str) -> Option<String> {
    None
}

#[cfg(feature = "desktop-clipboard")]
fn isolated_fork_set(content: &str) -> bool {
    let result = (|| -> color_eyre::Result<bool> {
        let mut child = Command::new(env::current_exe()?)
            .arg("--internal-clipboard-write")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(content.as_bytes())?;
        }
        Ok(child.wait()?.success())
    })();
    result.unwrap_or(false)
}
#[cfg(feature = "desktop-clipboard")]
fn fork_set(content: &str) -> bool {
    ForkClipboardContext::new()
        .and_then(|mut ctx| ctx.set_contents(content.to_owned()))
        .is_ok()
}
#[cfg(not(feature = "desktop-clipboard"))]
fn fork_set(_content: &str) -> bool {
    false
}

fn env_var_set(name: &str) -> bool {
    env::var(name).is_ok_and(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expires_only_the_copy_we_still_own() {
        assert!(clear_owned(
            "123456",
            || Some("123456".into()),
            |value| {
                assert!(value.is_empty());
                true
            }
        ));
        assert!(!clear_owned(
            "123456",
            || Some("user's newer copy".into()),
            |_| panic!("must retain newer clipboard")
        ));
        assert!(!clear_owned(
            "123456",
            || None,
            |_| panic!("must not erase unreadable clipboard")
        ));
    }
}
