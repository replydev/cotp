#![forbid(unsafe_code)]
use database_management::{
    get_elements, overwrite_database_json_key, overwrite_database_key, ReadResult,
};
use interface::app::AppResult;
use interface::event::{Event, EventHandler};
use interface::handler::handle_key_events;
use interface::ui::Tui;
use otp::otp_element::OTPDatabase;
use std::{io, vec};
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

fn init() -> Result<ReadResult, String> {
    match utils::create_db_if_needed() {
        Ok(needs_creation) => {
            if needs_creation {
                let mut pw = utils::prompt_for_passwords("Choose a password: ", 8, true);
                if database_management::overwrite_database_json("[]", &pw).is_err() {
                    return Err(String::from(
                        "An error occurred during database overwriting",
                    ));
                }
                pw.zeroize();
                return Ok((OTPDatabase::new(2, vec![]), vec![], vec![]));
            } else {
                get_elements()
            }
        }
        Err(()) => Err(String::from("An error occurred during database creation")),
    }
}

fn main() -> AppResult<()> {
    let mut result = match init() {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            std::process::exit(-1);
        }
    };
    match args::args_parser(&mut result.0) {
        // no args, show dashboard
        None => match dashboard(result) {
            Ok(()) => std::process::exit(0),
            Err(_) => std::process::exit(-2),
        },
        // args parsed, can exit
        Some(r) => match r {
            Ok(_) => match overwrite_database_key(&result.0, &result.1, &result.2) {
                Ok(_) => {
                    println!("Success");
                    std::process::exit(0)
                }
                Err(_) => {
                    eprintln!("An error occurred during database overwriting");
                    std::process::exit(-2)
                }
            },
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(-2);
            }
        },
    }
}

fn dashboard(read_result: ReadResult) -> AppResult<()> {
    let database = read_result.0;
    let mut key = read_result.1;
    let salt = read_result.2;

    if database.elements_ref().is_empty() {
        println!("No codes, type \"cotp -h\" to get help");
    } else {
        // Create an application.
        let mut app = interface::app::App::new(database);

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
            if overwrite_database_key(&app.database, &key, &salt).is_err() {
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

    Ok(())
}
