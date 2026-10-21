use super::{copy_selected, handle_exit, show_popup};
use crate::{
    interface::{
        app::{App, Popup},
        enums::{Focus, Page, PopupAction},
    },
    otp::otp_type::OTPType,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(super) fn main_handler(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        KeyCode::Char('a') => app.open_form(false),
        KeyCode::Char('e') => app.open_form(true),
        KeyCode::Esc => {
            if app.current_page == Page::Qrcode {
                app.reset();
            } else if !app.search_query.is_empty() {
                app.search_query.clear();
                app.filter();
            } else {
                handle_exit(app);
            }
        }
        KeyCode::Char('q' | 'Q') => handle_exit(app),
        KeyCode::Char('d' | 'D') | KeyCode::Delete if app.current_page == Page::Main => {
            if let Some(index) = app.table.selected_index() {
                let element = &app.database.elements_ref()[index];
                show_popup(
                    Popup {
                        text: format!(
                            "Remove {} / {} from your vault?\n\nThis removes the saved secret for this account. Changes are saved when you quit.",
                            element.issuer, element.label
                        ),
                        action: PopupAction::DeleteOtp,
                    },
                    app,
                );
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.reset();
            app.table.previous();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.reset();
            app.table.next();
        }
        KeyCode::Home => {
            app.reset();
            app.table.move_by(isize::MIN);
        }
        KeyCode::End => {
            app.reset();
            app.table.move_by(isize::MAX);
        }
        KeyCode::PageUp => {
            app.reset();
            app.table.move_by(-10);
        }
        KeyCode::PageDown => {
            app.reset();
            app.table.move_by(10);
        }
        KeyCode::Char('+') => handle_counter_switch(app, true),
        KeyCode::Char('-') => handle_counter_switch(app, false),
        KeyCode::Char('f') if key_event.modifiers.is_empty() && app.current_page == Page::Main => {
            if let Some(index) = app.table.selected_index() {
                let element = app
                    .database
                    .mut_element(index)
                    .expect("selected account exists");
                element.favorite = !element.favorite;
                let favorite = element.favorite;
                app.database.mark_modified();
                app.filter();
                app.notify(
                    if favorite {
                        "Added to favorites"
                    } else {
                        "Removed from favorites"
                    },
                    false,
                );
            }
        }
        KeyCode::Char('h') => {
            app.hide_codes = !app.hide_codes;
            app.reset();
        }
        KeyCode::Char(' ') => {
            if app.current_page == Page::Qrcode {
                app.reset();
            } else if app.table.selected_index().is_some() {
                app.current_page = Page::Qrcode;
            }
        }
        KeyCode::Char('?' | 'i') => show_popup(
            Popup {
                text: String::new(),
                action: PopupAction::GeneralInfo,
            },
            app,
        ),
        KeyCode::Char('f' | 'F') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.current_page = Page::Main;
            app.focus = Focus::SearchBar;
        }
        KeyCode::Char('/') => {
            app.current_page = Page::Main;
            app.focus = Focus::SearchBar;
        }
        KeyCode::Enter => copy_selected(app, app.current_page == Page::Qrcode),
        _ => {}
    }
}

fn handle_counter_switch(app: &mut App, increment: bool) {
    if let Some(selected) = app.table.selected_index()
        && let Some(element) = app.database.mut_element(selected)
        && element.type_ == OTPType::Hotp
    {
        let counter = element.counter.unwrap_or_default();
        let next = if increment {
            counter.saturating_add(1)
        } else {
            counter.saturating_sub(1)
        };
        element.counter = Some(next);
        app.database.mark_modified();
        app.tick(true);
        app.notify(format!("HOTP counter updated to {next}"), false);
    }
}
