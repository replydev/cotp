#![forbid(unsafe_code)]
use args::CotpArgs;
use clap::Parser;
use interface::app::AppResult;
use interface::event::{Event, EventHandler};
use interface::handler::handle_key_events;
use interface::ui::Tui;
use otp::otp_element::{OTPDatabase, CURRENT_DATABASE_VERSION};
use reading::{get_elements, ReadResult};
use std::{io, vec};
use tui::backend::CrosstermBackend;
use tui::Terminal;
use zeroize::Zeroize;

mod args;
mod argument_functions;
mod crypto;
mod exporters;
mod importers;
mod interface;
mod otp;
mod reading;
mod utils;

fn init() -> Result<ReadResult, String> {
    match utils::init_app() {
        Ok(first_run) => {
            if first_run {
                // Let's initialize the database file
                let mut pw = utils::verified_password("Choose a password: ", 8);
                let mut database = OTPDatabase {
                    version: CURRENT_DATABASE_VERSION,
                    elements: vec![],
                    ..Default::default()
                };
                let save_result = database.save_with_pw(&pw);
                pw.zeroize();
                match save_result {
                    Ok((key, salt)) => Ok((database, key, salt.to_vec())),
                    Err(_) => Err(String::from(
                        "An error occurred during database overwriting",
                    )),
                }
            } else {
                get_elements()
            }
        }
        Err(()) => Err(String::from("An error occurred during database creation")),
    }
}

fn main() -> AppResult<()> {
    let cotp_args = CotpArgs::parse();
    let (database, key, salt) = match init() {
        Ok(v) => v,
        Err(e) => {
            println!("{e}");
            std::process::exit(-1);
        }
    };

    let mut reowned_database = match args::args_parser(cotp_args, database) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("An error occurred: {e}");
            std::process::exit(-2)
        }
    };

    if reowned_database.is_modified() {
        match reowned_database.save(&key, &salt) {
            Ok(_) => {
                println!("Success");
            }
            Err(_) => {
                eprintln!("An error occurred during database overwriting");
                std::process::exit(-3)
            }
        }
    }
    std::process::exit(0)
}

fn dashboard(mut database: OTPDatabase) -> AppResult<OTPDatabase> {
    if database.elements_ref().is_empty() {
        println!("No codes, type \"cotp -h\" to get help");
    } else {
        // Create an application.
        let mut app = interface::app::App::new(&mut database);

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

        // Exit the user interface.
        tui.exit()?;
    }

    Ok(database)
}
