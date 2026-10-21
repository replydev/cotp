use crate::interface::{
    app::App,
    enums::{Focus, PopupAction},
};
use crossterm::event::{KeyCode, KeyEvent};

pub(super) fn popup_handler(key_event: KeyEvent, app: &mut App) {
    match app.popup.action {
        PopupAction::DeleteOtp => match key_event.code {
            KeyCode::Char('y' | 'Y') => {
                if let Some(index) = app.table.selected_index() {
                    app.database.delete_element(index);
                    // Database indices shift after deletion. Select the next visible account.
                    let position = app.table.state.selected().unwrap_or(0);
                    app.table.items.clear();
                    app.filter();
                    if !app.table.items.is_empty() {
                        app.table
                            .state
                            .select(Some(position.min(app.table.items.len() - 1)));
                    }
                    app.notify("Account removed · changes pending save", false);
                }
                app.focus = Focus::MainPage;
            }
            KeyCode::Char('n' | 'N') | KeyCode::Esc => app.focus = Focus::MainPage,
            _ => {}
        },
        PopupAction::GeneralInfo => match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => {
                app.popup_scroll = app.popup_scroll.saturating_add(1).min(14)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.popup_scroll = app.popup_scroll.saturating_sub(1)
            }
            KeyCode::Char('I' | 'i' | '?') | KeyCode::Esc | KeyCode::Enter => {
                app.focus = Focus::MainPage
            }
            _ => {}
        },
        PopupAction::SaveBeforeQuit => match key_event.code {
            KeyCode::Char('y' | 'Y') => app.running = false,
            KeyCode::Char('n' | 'N') => {
                app.database.needs_modification = false;
                app.running = false;
            }
            KeyCode::Esc => app.focus = Focus::MainPage,
            _ => {}
        },
    }
}
