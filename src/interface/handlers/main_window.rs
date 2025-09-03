use super::{
    super::enums::Page::{Main, Qrcode},
    copy_selected_code_to_clipboard,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    clipboard::copy_string_to_clipboard,
    interface::{
        app::{App, Popup},
        enums::{Focus, Page, PopupAction},
    },
    otp::otp_type::OTPType,
};

use super::{handle_exit, show_popup};

pub(super) fn main_handler(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        // exit application on ESC or Q
        KeyCode::Esc | KeyCode::Char('q' | 'Q') => {
            if app.focus != Focus::SearchBar {
                handle_exit(app);
            }
        }
        // exit application on Ctrl-D
        KeyCode::Char('d' | 'D' | 'c') => {
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
                );
            }
        }

        // Move into the table
        KeyCode::Up | KeyCode::Char('k') => {
            app.print_percentage = true;
            app.current_page = Main;
            app.table.previous();
        }

        KeyCode::Down | KeyCode::Char('j') => {
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

        KeyCode::Char(' ') => handle_switch_page(app, Qrcode),

        KeyCode::Char('?') => {
            let info_text = String::from(
                "
            Press:
            d -> Delete selected code
            + -> Increment the HOTP counter
            - -> Decrement the HOTP counter
            Space -> Show QRCode of the selected element
            Enter -> Copy the OTP Code to the clipboard
            CTRL-F | '/' -> Search codes
            CTRL-W | CTRL-U -> Clear the search query
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

        KeyCode::Char('f' | 'F') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.focus = Focus::SearchBar;
            }
        }

        KeyCode::Char('/') => app.focus = Focus::SearchBar,

        KeyCode::Enter => match app.current_page {
            Main => {
                app.label_text = copy_selected_code_to_clipboard(app);
                app.print_percentage = false;
            }
            Qrcode => {
                let selected_element = app
                    .table
                    .state
                    .selected()
                    .and_then(|index| app.database.elements_ref().get(index));

                if let Some(element) = selected_element {
                    let otp_uri = element.get_otpauth_uri();
                    let _ = copy_string_to_clipboard(&otp_uri);
                    app.qr_code_page_label = "OTP URI Copied to clipboard";
                }
            }
        },
        _ => {}
    }
}

fn handle_counter_switch(app: &mut App, increment: bool) {
    if let Some(selected) = app.table.state.selected()
        && let Some(element) = app.database.mut_element(selected)
        && element.type_ == OTPType::Hotp
    {
        // safe to unwrap because the element type is HOTP
        let counter = element.counter.unwrap();
        element.counter = if increment {
            Some(counter.saturating_add(1))
        } else {
            Some(counter.saturating_sub(1))
        };
        app.database.mark_modified();
        app.tick(true);
    }
}

fn handle_switch_page(app: &mut App, page: Page) {
    if app.current_page == page {
        app.reset();
    } else {
        app.current_page = page;
    }
}
