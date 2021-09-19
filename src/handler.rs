use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, AppResult};
use copypasta::{ClipboardContext, ClipboardProvider};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // exit application on ESC
        KeyCode::Esc => {
            app.running = false;
        }
        // exit application on Ctrl-D
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.running = false;
            }
        }
        // exit application on Q
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.running = false;
        }

        // Move into the table
        KeyCode::Up => {
            app.table.previous();
        }

        KeyCode::Down => {
            app.table.next();
        }

        KeyCode::Enter => {
            if let Some(selected) = app.table.state.selected(){
                if let Some(element) = app.table.items.get(selected){
                    if let Some(otp_code) = element.get(3){
                        let mut ctx = ClipboardContext::new().unwrap();
                        ctx.set_contents(otp_code.to_owned()).unwrap();
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}