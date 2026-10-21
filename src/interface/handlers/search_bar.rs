use super::copy_selected;
use crate::interface::{app::App, enums::Focus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(super) fn search_bar_handler(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        KeyCode::Char(c) if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            match c.to_ascii_lowercase() {
                'f' => app.focus = Focus::MainPage,
                'w' | 'u' => {
                    app.search_query.clear();
                    app.filter();
                }
                _ => {}
            }
        }
        KeyCode::Char(c) if !key_event.modifiers.contains(KeyModifiers::ALT) => {
            app.search_query.push(c);
            app.filter();
        }
        KeyCode::Enter => {
            copy_selected(app, false);
            app.focus = Focus::MainPage;
        }
        KeyCode::Esc | KeyCode::Tab => app.focus = Focus::MainPage,
        KeyCode::Backspace => {
            app.search_query.pop();
            app.filter();
        }
        KeyCode::Up => app.table.previous(),
        KeyCode::Down => app.table.next(),
        _ => {}
    }
}
