use crate::clipboard::{copy_string_to_clipboard, CopyType};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::interface::app::{App, AppResult};
use crate::interface::enums::Page::*;
use crate::otp::otp_type::OTPType;

use super::app::Popup;
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
    match app.popup.action {
        PopupAction::EditOtp => todo!(),
        PopupAction::DeleteOtp => match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Err(e) = delete_selected_code(app) {
                    app.popup.text = e;
                    return;
                }
                app.focus = Focus::MainPage;
                // Force table render
                app.tick(true);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.focus = Focus::MainPage;
            }
            _ => {}
        },
        PopupAction::GeneralInfo => match key_event.code {
            KeyCode::Char('I') | KeyCode::Char('i') | KeyCode::Esc | KeyCode::Enter => {
                app.focus = Focus::MainPage;
            }
            _ => {}
        },
        PopupAction::SaveBeforeQuit => match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.running = false;
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                app.database.needs_modification = false;
                app.running = false;
            }
            KeyCode::Esc => {
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
                    'c' | 'C' => handle_exit(app),
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
            app.focus = Focus::MainPage;
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
            handle_exit(app);
        }
        // exit application on Ctrl-D
        KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Char('c') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                handle_exit(app);
            } else if app.table.state.selected().is_some() {
                // Ask the user if he wants to delete the OTP Code
                show_popup(
                    Popup {
                        text: String::from("Do you want to delete the selected OTP Code? [Y/N]"),
                        percent_x: 60,
                        percent_y: 20,
                        action: PopupAction::DeleteOtp,
                    },
                    app,
                )
            }
        }
        // exit application on Q
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            if app.focus != Focus::SearchBar {
                handle_exit(app);
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

        KeyCode::Char('i') | KeyCode::Char('I') => {
            let info_text = String::from(
                "
            Press:
            d -> Delete selected code
            + -> Increment the HOTP counter
            - -> Decrement the HOTP counter
            k -> Show QRCode of the selected element
            Enter -> Copy the OTP Code to the clipboard
            CTRL-F -> Search codes
            CTRL-W -> Clear the search query
            q, CTRL-D, Esc -> Exit the application
            ",
            );
            show_popup(
                Popup {
                    text: info_text,
                    percent_x: 40,
                    percent_y: 50,
                    action: PopupAction::GeneralInfo,
                },
                app,
            );
        }

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
            if app.database.elements_ref().len() > selected {
                app.database.delete_element(selected);
                app.table.items.remove(selected);
                if selected >= app.database.elements_ref().len() {
                    app.table.previous();
                } else if app.database.elements_ref().is_empty() {
                    app.table.state.select(None)
                }
                Ok("Done".to_string())
            } else {
                Err("Index out of bounds".to_string())
            }
        }
        None => Err("No code selected".to_string()),
    }
}

fn copy_selected_code_to_clipboard(app: &mut App) -> String {
    match app.table.state.selected() {
        Some(selected) => match app.table.items.get(selected) {
            Some(element) => match element.values.get(3) {
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
            None => format!("Cannot fetch element from index: {selected}"),
        },
        None => "No code selected".to_string(),
    }
}

fn handle_counter_switch(app: &mut App, increment: bool) {
    if let Some(selected) = app.table.state.selected() {
        if let Some(element) = app.database.mut_element(selected) {
            if element.type_ == OTPType::Hotp {
                // safe to unwrap becouse the element type is HOTP
                let counter = element.counter.unwrap();
                element.counter = if increment {
                    Some(counter.checked_add(1).unwrap_or(u64::MAX))
                } else {
                    Some(counter.saturating_sub(1))
                };
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
    for iter in app.table.items.iter().enumerate() {
        let (index, row) = iter;
        if row
            .values
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
    for iter in app.table.items.iter().enumerate() {
        let (index, row) = iter;
        if row
            .values
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
    for iter in app.table.items.iter().enumerate() {
        let (index, row) = iter;
        if row
            .values
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
    for iter in app.table.items.iter().enumerate() {
        let (index, row) = iter;
        if row
            .values
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

fn show_popup(popup: Popup, app: &mut App) {
    app.focus = Focus::Popup;
    app.popup = popup;
}

fn handle_exit(app: &mut App) {
    if app.database.is_modified() {
        show_popup(
            Popup {
                text: String::from("Save changes? [Y/N]"),
                percent_x: 60,
                percent_y: 20,
                action: PopupAction::SaveBeforeQuit,
            },
            app,
        )
    } else {
        app.running = false;
    }
}
