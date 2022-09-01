#![forbid(unsafe_code)]
use database_management::overwrite_database_key;
use interface::app::AppResult;
use interface::event::{Event, EventHandler};
use interface::handler::handle_key_events;
use interface::ui::Tui;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use zeroize::Zeroize;

mod args;
mod argument_functions;
mod crypto;
mod database_management;
mod importers;
mod interface;
mod otp;
mod utils;

fn init() -> Result<bool, String> {
    match utils::create_db_if_needed() {
        Ok(value) => {
            if value {
                let mut pw = utils::prompt_for_passwords("Choose a password: ", 8, true);
                let result = match database_management::overwrite_database_json("[]", &pw) {
                    Ok(()) => Ok(true),
                    Err(_e) => Err(String::from(
                        "An error occurred during database overwriting",
                    )),
                };
                pw.zeroize();
                return result;
            }
            Ok(false)
        }
        Err(()) => Err(String::from("An error occurred during database creation")),
    }
}

fn main() -> AppResult<()> {
    match init() {
        Ok(true) => {
            println!("Database correctly initialized");
            return Ok(());
        }
        Ok(false) => {}
        Err(e) => {
            println!("{}", e);
            std::process::exit(-1);
        }
    }
    match args::args_parser() {
        // no args, show dashboard
        true => match dashboard() {
            Ok(()) => std::process::exit(0),
            Err(_) => std::process::exit(-2),
        },
        // args parsed, can exit
        false => std::process::exit(0),
    }
}

fn dashboard() -> AppResult<()> {
    match database_management::get_elements() {
        Ok((elements, mut key, salt)) => {
            if elements.is_empty() {
                println!("No codes, type \"cotp -h\" to get help");
            } else {
                // Create an application.
                let mut app = interface::app::App::new(elements);

                // Initialize the terminal user interface.
                let backend = CrosstermBackend::new(io::stderr());
                let terminal = Terminal::new(backend)?;
                let events = EventHandler::new(250);
                let mut tui = Tui::new(terminal, events);
                tui.init()?;

                // Start the main loop.
                while app.running {
                    // Render the user interface.
                    tui.draw(&mut app)?;
                    // Handle events.
                    match tui.events.next()? {
                        Event::Tick => app.tick(false),
                        Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
                        Event::Mouse(_) => {}
                        Event::Resize(_, _) => {}
                        Event::FocusGained() => {}
                        Event::FocusLost() => {}
                        Event::Paste(_) => {}
                    }
                }

                // Overwrite database if modified
                let error: Option<String> = if app.data_modified {
                    if let Err(_) = overwrite_database_key(&app.elements, &key, &salt) {
                        Some("Failed to overwrite database".to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Zeroize the key
                key.zeroize();

                // Print the error
                if error.is_some() {
                    eprintln!("{}", error.unwrap());
                }

                // Exit the user interface.
                tui.exit()?;
            }
        }
        Err(e) => {
            eprintln!("An error occurred: {}", e);
            return Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, e)));
        }
    }
    Ok(())
}
