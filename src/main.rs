#![forbid(unsafe_code)]
use arguments::{CotpArgs, args_parser};
use clap::Parser;
use interface::app::AppResult;
use interface::event::{Event, EventHandler};
use interface::handlers::handle_key_events;
use interface::ui::Tui;
use otp::otp_element::{CURRENT_DATABASE_VERSION, OTPDatabase};
use path::init_path;
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use reading::{ReadResult, get_elements_from_input, get_elements_from_stdin};
use std::process::ExitCode;
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

fn init(args: &CotpArgs) -> eyre::Result<ReadResult> {
    init_path(args);

    let first_run = utils::init_app()?;
    if first_run {
        // Let's initialize the database file
        let mut pw = utils::try_verified_password("Choose a password: ", 8)?;
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

fn main() -> ExitCode {
    let cotp_args: CotpArgs = CotpArgs::parse();
    let (database, mut key, salt) = match init(&cotp_args) {
        Ok(v) => v,
        Err(e) => {
            // "{e:#}" prints the whole eyre error chain, e.g.
            // "outer context: root cause", keeping the root cause visible.
            eprintln!("An error occurred: {e:#}");
            return ExitCode::from(1);
        }
    };

    let mut reowned_database = match args_parser(cotp_args, database) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("An error occurred: {e:#}");
            key.zeroize();
            return ExitCode::from(2);
        }
    };

    let exit_code = if reowned_database.is_modified() {
        match reowned_database.save(&key, &salt) {
            Ok(()) => {
                println!("Modifications have been persisted");
                ExitCode::SUCCESS
            }
            _ => {
                eprintln!("An error occurred during database overwriting");
                ExitCode::from(1)
            }
        }
    } else {
        ExitCode::SUCCESS
    };
    key.zeroize();
    exit_code
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
                Event::Key(key_event) => handle_key_events(key_event, &mut app),
            }
        }

        // Exit the user interface.
        tui.exit()?;
    }

    Ok(database)
}
