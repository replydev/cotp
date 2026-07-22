use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::interface::{app::App, enums::Focus};

use super::{copy_selected_code_to_clipboard, handle_exit, main_window::main_handler};

pub(super) fn search_bar_handler(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        KeyCode::Char(c) => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                match c.to_ascii_lowercase() {
                    'f' => {
                        app.search_query.clear();
                        app.focus = Focus::MainPage;
                    }
                    'c' => handle_exit(app),
                    'w' | 'u' => app.search_query.clear(),
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
    let query = app.search_query.to_lowercase();
    // Single ranked pass over the rows: an issuer prefix match wins over a
    // label prefix match, which wins over an issuer substring match, which
    // wins over a label substring match; ties are broken by row order
    let best_match = app
        .table
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            let issuer = row.issuer.to_lowercase();
            let label = row.label.to_lowercase();
            let rank = if issuer.starts_with(&query) {
                0
            } else if label.starts_with(&query) {
                1
            } else if issuer.contains(&query) {
                2
            } else if label.contains(&query) {
                3
            } else {
                return None;
            };
            Some((rank, index))
        })
        .min_by_key(|&(rank, index)| (rank, index));

    if let Some((_, index)) = best_match {
        app.table.state.select(Some(index));
    }
    // TODO Handle if no search results
}
