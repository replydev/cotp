use std::{env, io};

use sodiumoxide;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use interface::app::AppResult;
use interface::event::{Event, EventHandler};
use interface::handler::handle_key_events;
use otp::otp_helper;
use interface::ui::Tui;
use zeroize::Zeroize;
use clap;
use clap::{App, AppSettings, ArgMatches, Arg};

mod utils;
mod argument_functions;
mod cryptography;
mod importers;
mod otp;
mod interface;
mod database_management;

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
                let mut pw = cryptography::prompt_for_passwords("Choose a password: ", 8, true);
                let result = match database_management::overwrite_database_json("[]", &pw) {
                    Ok(()) => Ok(true),
                    Err(_e) => Err(String::from("An error occurred during database overwriting")),
                };
                pw.zeroize();
                return result;
            }
            Ok(false)
        }
        Err(()) => {
            return Err(String::from("An error occurred during database creation"));
        }
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
    match args_parser() {
        Ok(_) => std::process::exit(0),
        Err(_) => std::process::exit(-2),
    }
}

fn dashboard() -> AppResult<()> {
    match otp_helper::read_codes() {
        Ok(elements) => {
            if elements.len() == 0 {
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

fn args_parser() -> AppResult<()> {
    match get_matches().subcommand() {
        Some(("add",add_matches)) => argument_functions::add(add_matches),
        Some(("edit",edit_matches)) => argument_functions::edit(edit_matches),
        Some(("remove",remove_matches)) => argument_functions::remove(remove_matches),
        Some(("import",import_matches)) => argument_functions::import(import_matches),
        Some(("info",info_matches)) => argument_functions::info(info_matches),
        Some(("export",_)) => argument_functions::export(),
        Some(("passwd",_)) => argument_functions::change_password(),
        _ => return dashboard(),
    }

    AppResult::Ok(())
}

fn get_matches() -> ArgMatches{
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS").split(',').next().unwrap_or("replydev <commoncargo@tutanota.com>"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .license("GPL3")
        .subcommand(
            App::new("add")
                .about("Add a new OTP Code")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    Arg::new("issuer")
                    .short('i')
                    .long("issuer")
                    .about("OTP Code issuer")
                    .takes_value(true)
                    .required(false)
                    .default_value("")
                )
                .arg(
                    Arg::new("label")
                    .short('l')
                    .long("label")
                    .about("OTP Code label")
                    .takes_value(true)
                    .required(true)
                )
                .arg(
                    Arg::new("algorithm")
                    .short('a')
                    .long("algoritmh")
                    .about("OTP Code algorithm")
                    .takes_value(true)
                    .required(false)
                    .possible_values(&["SHA1","SHA256","SHA512"])
                    .default_value("SHA1")
                )
                .arg(
                    Arg::new("digits")
                    .short('d')
                    .long("digits")
                    .about("OTP Code digits")
                    .takes_value(true)
                    .required(false)
                    .default_value("6")
                )
        )
        .subcommand(
            App::new("edit")
                .about("Edit an OTP code")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    Arg::new("index")
                    .short('n')
                    .long("index")
                    .about("OTP Code index")
                    .takes_value(true)
                    .required(true)
                )
                .arg(
                    Arg::new("issuer")
                    .short('i')
                    .long("issuer")
                    .about("OTP Code issuer")
                    .takes_value(true)
                    .required_unless_present_any(["label","algorithm","digits"])
                )
                .arg(
                    Arg::new("label")
                    .short('l')
                    .long("label")
                    .about("OTP Code label")
                    .takes_value(true)
                    .required_unless_present_any(["issuer","algorithm","digits"])
                )
                .arg(
                    Arg::new("algorithm")
                    .short('a')
                    .long("algoritmh")
                    .about("OTP Code algorithm")
                    .takes_value(true)
                    .required_unless_present_any(["label","issuer","digits"])
                    .possible_values(&["SHA1","SHA256","SHA512"])
                )
                .arg(
                    Arg::new("digits")
                    .short('d')
                    .long("digits")
                    .about("OTP Code digits")
                    .takes_value(true)
                    .required_unless_present_any(["label","algorithm","issuer"])
                )
                .arg(
                    Arg::new("change-secret")
                    .short('c')
                    .long("change-secret")
                    .about("Change the OTP code secret")
                    .takes_value(false)
                )
        )
        .subcommand(
            App::new("remove")
                .about("Remove an OTP code")
                .setting(AppSettings::ArgRequiredElseHelp) // They can even have different settings
                .arg(
                    Arg::new("index")
                        .short('n')
                        .long("index")
                        .about("OTP code index")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("import")
                .about("Import from backups")
                .setting(AppSettings::ArgRequiredElseHelp) // They can even have different settings
                .arg(
                    Arg::new("appname")
                        .short('a')
                        .long("appname")
                        .about("App from which you are importing the backup")
                        .possible_values(&["cotp","andotp","aegis","gauth","authy"])
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::new("path")
                        .short('p')
                        .long("path")
                        .about("Backup path")
                        .takes_value(true)
                        .required(true),
                )
                
        )
        .subcommand(
            App::new("export")
                .about("Export your database")
        )
        .subcommand(
            App::new("info")
                .about("Show OTP code information")
                .setting(AppSettings::ArgRequiredElseHelp) // They can even have different settings
                .arg(
                    Arg::new("index")
                        .short('n')
                        .long("index")
                        .about("OTP code index")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("passwd")
            .about("Change your database password")
        )
        .get_matches()
}
