use std::{env, io};

use sodiumoxide;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use app::{App, AppResult};
use event::{Event, EventHandler};
use handler::handle_key_events;
use otp::otp_helper;
use ui::Tui;

mod database_loader;
mod utils;
mod argument_functions;
mod cryptography;
mod importers;
mod otp;
mod ui;
mod event;
mod app;
mod handler;
mod table;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_title() {
    println!("cotp v{}", VERSION);
    println!("written by @replydev\n");
    #[cfg(debug_assertions)]
    println!("****DEBUG VERSION****\n");
}

fn init() -> Result<bool, String> {
    match sodiumoxide::init() {
        Err(()) => {
            return Err(String::from("Error during sodiumoxide initialization"));
        }
        _ => {}
    };
    match utils::create_db_if_needed() {
        Ok(value) => {
            if value {
                let pw = &cryptography::prompt_for_passwords("Choose a password: ", 8, true);
                return match database_loader::overwrite_database_json("[]", pw) {
                    Ok(()) => Ok(true),
                    Err(_e) => Err(String::from("An error occurred during database overwriting")),
                };
            }
            Ok(false)
        }
        Err(()) => {
            return Err(String::from("An error occurred during database creation"));
        }
    }
}

fn main() -> AppResult<()> {
    print_title();

    let init_result = init();
    match init_result {
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
    let args: Vec<String> = env::args().collect();
    if !args_parser(args) {
        match dashboard() {
            Ok(_) => std::process::exit(0),
            Err(_) => std::process::exit(-2),
        }
    }
    Ok(())
}

fn dashboard() -> AppResult<()> {
    match otp_helper::read_codes() {
        Ok(elements) => {
            if elements.len() == 0 {
                println!("No codes, type \"cotp -h\" to get help");
            } else {
                // Create an application.
                let mut app = App::new(elements);

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
                        Event::Tick => app.tick(),
                        Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
                        Event::Mouse(_) => {}
                        Event::Resize(_, _) => {}
                    }
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

fn args_parser(args: Vec<String>) -> bool {
    if args.len() == 1 {
        return false;
    }

    match &args[1][..] {
        "-i" | "--import" => argument_functions::import(args),
        "-h" | "--help" => argument_functions::help(),
        "-a" | "--add" => argument_functions::add(args),
        "-r" | "--remove" => argument_functions::remove(args),
        "-e" | "--edit" => argument_functions::edit(args),
        "-ex" | "--export" => argument_functions::export(args),
        "-j" | "--json" => argument_functions::json(args),
        "-s" | "--single" => argument_functions::single(args),
        "-in" | "--info" => argument_functions::info(args),
        "-chpw" | "--change-password" => argument_functions::change_password(args),
        _ => {
            println!("Invalid argument: {}, type cotp -h to get command options", args[1]);
            std::process::exit(-1);
        }
    }
    true
}
