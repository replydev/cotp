use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::interface::app::{App, AppResult};
use crate::interface::page::Page::*;
use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;

use super::page::Page;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if !app.search_bar_focused {
        handle_key_events_main(key_event, app);
    } else {
        match key_event.code {
            KeyCode::Char(c) => {
                if key_event.modifiers == KeyModifiers::CONTROL && c == 'f' {
                    app.current_page = Main;
                } else {
                    app.search_query.push(c);
                }
            }
            KeyCode::Enter => {
                app.search_bar_focused = false;
                search_and_select(app);
                app.search_query.clear();
            }
            KeyCode::Esc => {
                app.search_bar_focused = false;
            }
            KeyCode::Backspace => {
                app.search_query.pop();
            }
            KeyCode::Up | KeyCode::Down => {
                app.search_bar_focused = false;
                handle_key_events_main(key_event, app);
            }
            _ => {}
        }
    }

    Ok(())
}

fn handle_key_events_main(key_event: KeyEvent, app: &mut App) {
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
            // Just could do: app.running = app.search_bar_focused;
            // But this is more readable
            if !app.search_bar_focused {
                app.running = false;
            }
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

        KeyCode::Char('k') | KeyCode::Char('K') => handle_switch_page(app, Qrcode),

        KeyCode::Char('i') | KeyCode::Char('I') => handle_switch_page(app, Info),

        KeyCode::Char('f') | KeyCode::Char('F') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.search_bar_focused = true;
            }
        }

        KeyCode::Char('/') => app.search_bar_focused = true,

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

fn handle_switch_page(app: &mut App, page: Page) {
    let default_page = Main;
    if app.current_page == page {
        app.current_page = default_page;
    } else {
        app.current_page = page;
    }
}

fn search_and_select(app: &mut App) {
    // Check for issuer
    for row in app.table.items.iter().enumerate() {
        let (index, values) = row;
        if values
            .get(1)
            .unwrap()
            .to_lowercase()
            .contains(&app.search_query.to_lowercase())
        {
            app.table.state.select(Some(index));
            return;
        }
    }
    // Check for label
    for row in app.table.items.iter().enumerate() {
        let (index, values) = row;
        if values
            .get(2)
            .unwrap()
            .to_lowercase()
            .contains(&app.search_query.to_lowercase())
        {
            app.table.state.select(Some(index));
            return;
        }
    }
    // TODO Handle if no search results
}
