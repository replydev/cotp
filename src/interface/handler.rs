use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::crypto::cryptography::gen_salt;
use crate::database_management::overwrite_database_key;
use crate::interface::app::{App, AppResult};
use crate::interface::enums::Page::*;
use crate::utils::{copy_string_to_clipboard, CopyType};

use super::enums::Page;
use super::enums::{Focus, PopupAction};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.focus {
        Focus::MainPage => main_handler(key_event, app),
        Focus::SearchBar => search_bar_handler(key_event, app),
        Focus::Popup => popup_handler(key_event, app),
    }
    Ok(())
}

fn popup_handler(key_event: KeyEvent, app: &mut App) {
    match app.popup_action {
        PopupAction::EditOtp => todo!(),
        PopupAction::DeleteOtp => match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Err(e) = delete_selected_code(app) {
                    app.popup_text = e;
                    return;
                }
                let key = app.data_key.clone();
                let salt = gen_salt().unwrap();
                match overwrite_database_key(&app.elements, key, &salt) {
                    Ok(_) => {
                        app.focus = Focus::MainPage;
                        app.popup_text = String::from("Done");
                        // Force table render
                        app.tick(true);
                    }
                    Err(e) => app.popup_text = e.to_string(),
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                app.focus = Focus::MainPage;
            }
            _ => {}
        },
    }
}

fn search_bar_handler(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        KeyCode::Char(c) => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                match c {
                    'f' | 'F' => {
                        app.search_query.clear();
                        app.focus = Focus::MainPage;
                    }
                    'c' | 'C' => app.running = false,
                    'w' | 'W' => app.search_query.clear(),
                    _ => {}
                }
            } else {
                app.search_query.push(c);
                search_and_select(app);
            }
        }
        KeyCode::Enter => {
            app.label_text = copy_selected_code_to_clipboard(app);
            app.print_percentage = false;
        }
        KeyCode::Esc => {
            app.focus = Focus::MainPage;
        }
        KeyCode::Backspace => {
            app.search_query.pop();
        }
        KeyCode::Up | KeyCode::Down => {
            app.focus = Focus::MainPage;
            main_handler(key_event, app);
        }
        _ => {}
    }
}

fn main_handler(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        // exit application on ESC
        KeyCode::Esc => {
            app.running = false;
        }
        // exit application on Ctrl-D
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.running = false;
            } else if app.table.state.selected().is_some() {
                // Ask the user if he wants to delete the OTP Code
                app.focus = Focus::Popup;
                app.popup_text = String::from("Do you want to delete the selected OTP Code?");
                app.popup_action = PopupAction::DeleteOtp;
            }
        }
        // exit application on Q
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            if app.focus != Focus::SearchBar {
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
                app.focus = Focus::SearchBar;
            }
        }

        KeyCode::Char('/') => app.focus = Focus::SearchBar,

        KeyCode::Enter => {
            app.label_text = copy_selected_code_to_clipboard(app);
            app.print_percentage = false;
        }
        _ => {}
    }
}

fn delete_selected_code(app: &mut App) -> Result<String, String> {
    match app.table.state.selected() {
        Some(selected) => {
            app.elements.remove(selected);

            Ok("Done".to_string())
        }
        None => Err("No code selected".to_string()),
    }
}

fn copy_selected_code_to_clipboard(app: &mut App) -> String {
    match app.table.state.selected() {
        Some(selected) => match app.table.items.get(selected) {
            Some(element) => match element.get(3) {
                Some(otp_code) => {
                    if let Ok(result) = copy_string_to_clipboard(otp_code.to_owned()) {
                        match result {
                            CopyType::Native => "Copied!".to_string(),
                            CopyType::OSC52 => "Remote copied!".to_string(),
                        }
                    } else {
                        "Cannot copy".to_string()
                    }
                }
                None => "Cannot get OTP Code column".to_string(),
            },
            None => format!("Cannot fetch element from index: {}", selected),
        },
        None => "No code selected".to_string(),
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
            .starts_with(&app.search_query.to_lowercase())
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
            .starts_with(&app.search_query.to_lowercase())
        {
            app.table.state.select(Some(index));
            return;
        }
    }
    // Check if issuer contains the query
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
    // Check if label contains the query
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
