#![forbid(unsafe_code)]
use arguments::{args_parser, CotpArgs};
use clap::Parser;
use color_eyre::eyre::eyre;
use interface::app::AppResult;
use interface::event::{Event, EventHandler};
use interface::handlers::handle_key_events;
use interface::ui::Tui;
use otp::otp_element::{OTPDatabase, CURRENT_DATABASE_VERSION};
use path::init_path;
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use reading::{get_elements_from_input, get_elements_from_stdin, ReadResult};
use std::{io, vec};
use zeroize::Zeroize;

mod arguments;
mod clipboard;
mod crypto;
mod exporters;
mod importers;
mod interface;
mod otp;
mod path;
mod reading;
mod utils;

fn init(args: &CotpArgs) -> color_eyre::Result<ReadResult> {
    init_path(args);

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
                save_result.map(|(key, salt)| (database, key, salt.to_vec()))
            } else if args.password_from_stdin {
                get_elements_from_stdin()
            } else {
                get_elements_from_input()
            }
        }
        Err(()) => Err(eyre!("An error occurred during database creation")),
    }
}

fn main() -> AppResult<()> {
    color_eyre::install()?;

    let cotp_args: CotpArgs = CotpArgs::parse();
    let (database, mut key, salt) = match init(&cotp_args) {
        Ok(v) => v,
        Err(e) => {
            println!("{e}");
            std::process::exit(-1);
        }
    };

    let mut reowned_database = match args_parser(cotp_args, database) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("An error occurred: {e}");
            key.zeroize();
            std::process::exit(-2)
        }
    };

    let error_code = if reowned_database.is_modified() {
        match reowned_database.save(&key, &salt) {
            Ok(_) => {
                println!("Modifications has been persisted");
                0
            }
            Err(_) => {
                eprintln!("An error occurred during database overwriting");
                -1
            }
        }
    } else {
        0
    };
    key.zeroize();
    std::process::exit(error_code)
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
