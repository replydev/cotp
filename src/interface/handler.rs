use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::interface::app::{App, AppResult};
use crate::interface::page::Page::{Info, Main, Qrcode};
use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // exit application on ESC
        KeyCode::Esc => {
            app.running = false;
        }
        // exit application on Ctrl-D
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.running = false;
            }
        }
        // exit application on Q
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.running = false;
        }

        // Move into the table
        KeyCode::Up => {
            app.print_percentage = true;
            app.current_page = Main;
            app.table.previous();
        }

        KeyCode::Down => {
            app.print_percentage = true;
            app.current_page = Main;
            app.table.next();
        }

        KeyCode::Char('+') => {
            app.current_page = Main;
            handle_counter_switch(app, true);
        }

        KeyCode::Char('-') => {
            app.current_page = Main;
            handle_counter_switch(app, false);
        }

        KeyCode::Char('k') | KeyCode::Char('K') => {
            if app.current_page == Qrcode {
                app.current_page = Main
            } else {
                app.current_page = Qrcode;
            }
        }

        KeyCode::Char('i') | KeyCode::Char('I') => {
            if app.current_page == Info {
                app.current_page = Main
            } else {
                app.current_page = Info;
            }
        }

        KeyCode::Enter => {
            if let Some(selected) = app.table.state.selected() {
                if let Some(element) = app.table.items.get(selected) {
                    if let Some(otp_code) = element.get(3) {
                        // in some occasions we can't copy contents to clipboard, so let's check for a good result
                        if let Ok(mut ctx) = ClipboardContext::new() {
                            match ctx.set_contents(otp_code.to_owned()) {
                                Ok(_) => app.label_text = String::from("Copied!"),
                                Err(_) => app.label_text = String::from("Cannot copy"),
                            }
                            app.print_percentage = false;
                            app.current_page = Main;
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_counter_switch(app: &mut App, increment: bool) {
    if let Some(selected) = app.table.state.selected() {
        if let Some(element) = app.elements.get_mut(selected) {
            if element.type_().to_uppercase() == "HOTP" {
                // safe to unwrap becouse the element type is HOTP
                let counter = element.counter().unwrap();
                element.set_counter(if increment {
                    Some(counter.checked_add(1).unwrap_or(u64::MAX))
                } else {
                    Some(counter.saturating_sub(1))
                });
                app.tick(true);
            }
        }
    }
}
