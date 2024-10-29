use crossterm::event::{KeyCode, KeyEvent};

use crate::interface::{
    app::App,
    enums::{Focus, PopupAction},
};

pub(super) fn popup_handler(key_event: KeyEvent, app: &mut App) {
    match app.popup.action {
        PopupAction::EditOtp => todo!(),
        PopupAction::DeleteOtp => match key_event.code {
            KeyCode::Char('y' | 'Y') => {
                if let Err(e) = delete_selected_code(app) {
                    app.popup.text = e;
                    return;
                }
                app.focus = Focus::MainPage;
                // Force table render
                app.tick(true);
            }
            KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                app.focus = Focus::MainPage;
            }
            _ => {}
        },
        PopupAction::GeneralInfo => match key_event.code {
            KeyCode::Char('I' | 'i') | KeyCode::Esc | KeyCode::Enter => {
                app.focus = Focus::MainPage;
            }
            _ => {}
        },
        PopupAction::SaveBeforeQuit => match key_event.code {
            KeyCode::Char('y' | 'Y') => {
                app.running = false;
            }
            KeyCode::Char('n' | 'N') => {
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

fn delete_selected_code(app: &mut App) -> Result<String, String> {
    match app.table.state.selected() {
        Some(selected) => {
            if app.database.elements_ref().len() > selected {
                app.database.delete_element(selected);
                app.table.items.remove(selected);
                if selected >= app.database.elements_ref().len() {
                    app.table.previous();
                } else if app.database.elements_ref().is_empty() {
                    app.table.state.select(None);
                }
                Ok("Done".to_string())
            } else {
                Err("Index out of bounds".to_string())
            }
        }
        None => Err("No code selected".to_string()),
    }
}
