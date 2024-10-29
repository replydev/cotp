use crossterm::event::KeyEvent;

use crate::clipboard::{copy_string_to_clipboard, CopyType};

use self::{main_window::main_handler, popup::popup_handler, search_bar::search_bar_handler};

use super::{
    app::{App, Popup},
    enums::{Focus, PopupAction},
};

mod main_window;
mod popup;
mod search_bar;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) {
    match app.focus {
        Focus::MainPage => main_handler(key_event, app),
        Focus::SearchBar => search_bar_handler(key_event, app),
        Focus::Popup => popup_handler(key_event, app),
    }
}

pub(super) fn handle_exit(app: &mut App) {
    if app.database.is_modified() {
        show_popup(
            Popup {
                text: String::from("Save changes? [Y/N]"),
                percent_x: 60,
                percent_y: 20,
                action: PopupAction::SaveBeforeQuit,
            },
            app,
        );
    } else {
        app.running = false;
    }
}

pub(crate) fn copy_selected_code_to_clipboard(app: &mut App) -> String {
    match app.table.state.selected() {
        Some(selected) => match app.table.items.get(selected) {
            Some(element) => match element.values.get(3) {
                Some(otp_code) => {
                    if let Ok(result) = copy_string_to_clipboard(otp_code) {
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

fn show_popup(popup: Popup, app: &mut App) {
    app.focus = Focus::Popup;
    app.popup = popup;
}
