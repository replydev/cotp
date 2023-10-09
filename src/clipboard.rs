use base64::{engine::general_purpose, Engine as _};
use copypasta_ext::prelude::*;
use copypasta_ext::wayland_bin::WaylandBinClipboardContext;
use copypasta_ext::x11_bin::ClipboardContext as BinClipboardContext;
use copypasta_ext::x11_fork::ClipboardContext as ForkClipboardContext;
use crossterm::style::Print;
use std::{env, io};

pub enum CopyType {
    Native,
    OSC52,
}

pub fn copy_string_to_clipboard(content: String) -> Result<CopyType, ()> {
    if ssh_clipboard(content.as_str()) {
        Ok(CopyType::OSC52)
    } else if wayland_clipboard(content.as_str()) || other_platform_clipboard(content.as_str()) {
        Ok(CopyType::Native)
    } else {
        Err(())
    }
}

fn ssh_clipboard(content: &str) -> bool {
    env_var_set("SSH_CONNECTION")
        // We do not use copypasta_ext::osc52 module because we have enabled terminal raw mode, so we print with crossterm utilities
        // Check https://github.com/timvisee/rust-clipboard-ext/blob/371df19d2f961882a21c957f396d1e24548d1f28/src/osc52.rs#L92
        && crossterm::execute!(
            io::stdout(),
            Print(format!(
                "\x1B]52;c;{}\x07",
                general_purpose::STANDARD.encode(content)
            ))
        )
        .is_ok()
}

fn wayland_clipboard(content: &str) -> bool {
    env_var_set("WAYLAND_DISPLAY")
        && WaylandBinClipboardContext::new()
            .and_then(|mut ctx| ctx.set_contents(content.to_owned()))
            .is_ok()
}

fn other_platform_clipboard(content: &str) -> bool {
    BinClipboardContext::new()
        .and_then(|mut ctx| ctx.set_contents(content.to_owned()))
        .is_ok()
        || ForkClipboardContext::new()
            .and_then(|mut ctx| ctx.set_contents(content.to_owned()))
            .is_ok()
}

fn env_var_set(env_var: &str) -> bool {
    env::var(env_var)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}
