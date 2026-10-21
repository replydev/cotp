use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use self::{main_window::main_handler, popup::popup_handler, search_bar::search_bar_handler};
use super::{
    app::{App, Popup},
    enums::{Action, Focus, Page, PopupAction},
    row::code_for,
};
use crate::clipboard::{CopyType, copy_string_to_clipboard};

mod main_window;
mod popup;
mod search_bar;

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) {
    if app.focus == Focus::Form {
        super::form::handle_form_key(key_event, app);
        return;
    }
    if key_event.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key_event.code, KeyCode::Char('c' | 'd'))
    {
        handle_exit(app);
        return;
    }
    match app.focus {
        Focus::MainPage => main_handler(key_event, app),
        Focus::SearchBar => search_bar_handler(key_event, app),
        Focus::Popup => popup_handler(key_event, app),
        Focus::Form => super::form::handle_form_key(key_event, app),
    }
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) {
    if event.kind == MouseEventKind::Down(MouseButton::Left) {
        let action = app
            .hitboxes
            .iter()
            .rev()
            .find(|(area, _)| area.contains((event.column, event.row).into()))
            .map(|(_, action)| *action);
        match action {
            Some(Action::Key(code)) => {
                handle_key_events(KeyEvent::new(code, KeyModifiers::NONE), app)
            }
            Some(Action::Command(code)) => {
                app.focus = Focus::MainPage;
                main_handler(KeyEvent::new(code, KeyModifiers::NONE), app);
            }
            Some(Action::ClearSearch) => {
                app.search_query.clear();
                app.filter();
            }
            Some(Action::Search) => {
                app.current_page = Page::Main;
                app.focus = Focus::SearchBar;
            }
            Some(Action::Select(index)) => {
                app.table.state.select(Some(index));
                app.focus = Focus::MainPage;
            }
            Some(Action::Field(index)) => {
                if let Some(form) = app.form.as_mut() {
                    form.active = index;
                    if super::form::AccountForm::is_choice(index) {
                        form.cycle(false);
                    }
                }
            }
            Some(Action::Save) => app.save_form(),
            Some(Action::Cancel) => app.cancel_form(),
            None => {}
        }
        return;
    }
    let backwards = match event.kind {
        MouseEventKind::ScrollUp => true,
        MouseEventKind::ScrollDown => false,
        _ => return,
    };
    match app.focus {
        Focus::Form => {
            if let Some(form) = app.form.as_mut() {
                form.navigate(backwards);
            }
        }
        Focus::Popup => {
            popup_handler(
                KeyEvent::new(
                    if backwards {
                        KeyCode::Up
                    } else {
                        KeyCode::Down
                    },
                    KeyModifiers::NONE,
                ),
                app,
            );
        }
        _ if app.current_page == Page::Main => {
            if backwards {
                app.table.previous();
            } else {
                app.table.next();
            }
        }
        _ => {}
    }
}

pub fn handle_paste(text: &str, app: &mut App) {
    if let Some(form) = app.form.as_mut() {
        form.paste(text);
        return;
    }
    if app.focus == Focus::SearchBar {
        app.search_query
            .extend(text.chars().filter(|c| !c.is_control()));
        app.filter();
    }
}

pub(super) fn handle_exit(app: &mut App) {
    if app.database.is_modified() {
        show_popup(Popup {
            text: "Your account changes have not been saved.\n\nSave them to your encrypted vault before quitting?".into(),
            action: PopupAction::SaveBeforeQuit,
        }, app);
    } else {
        app.running = false;
    }
}

pub(crate) fn copy_selected(app: &mut App, uri: bool) {
    let result = app
        .table
        .selected_index()
        .ok_or_else(|| "No account selected".to_string())
        .and_then(|index| {
            let element = app
                .database
                .get_element(index)
                .ok_or_else(|| "No account selected".to_string())?;
            if uri {
                Ok(element.get_otpauth_uri())
            } else if element.type_ == crate::otp::otp_type::OTPType::Hotp {
                crate::persistence::consume_hotp(app.database, index)
                    .map_err(|error| error.to_string())
            } else {
                code_for(element)
            }
        })
        .and_then(|text| copy_string_to_clipboard(&text).map_err(|error| error.to_string()));
    let noun = if uri { "OTP URI" } else { "Code" };
    match result {
        Ok(CopyType::Native) => app.notify(format!("✓ {noun} copied to clipboard"), false),
        Ok(CopyType::OSC52) => app.notify(format!("✓ {noun} sent to terminal clipboard"), false),
        Err(error) => app.notify(format!("Could not copy: {error}"), true),
    }
}

fn show_popup(popup: Popup, app: &mut App) {
    app.focus = Focus::Popup;
    app.popup = popup;
    app.popup_scroll = 0;
}
