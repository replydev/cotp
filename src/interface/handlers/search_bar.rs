use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::interface::{app::App, enums::Focus};

use super::{copy_selected_code_to_clipboard, handle_exit, main_window::main_handler};

pub(super) fn search_bar_handler(key_event: KeyEvent, app: &mut App) {
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
