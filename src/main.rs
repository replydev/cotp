#![forbid(unsafe_code)]
use arguments::{CotpArgs, args_parser};
use clap::Parser;
use color_eyre::eyre::eyre;
use interface::app::AppResult;
use interface::event::{Event, EventHandler};
use interface::handlers::{handle_key_events, handle_mouse_events, handle_paste};
use interface::ui::Tui;
use otp::otp_element::{CURRENT_DATABASE_VERSION, OTPDatabase};
use path::init_path;
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use reading::{ReadResult, get_elements_from_input, get_elements_from_stdin};
use std::{
    io::{self, IsTerminal},
    vec,
};
use zeroize::Zeroize;

mod arguments;
mod backup;
mod clipboard;
mod crypto;
mod database_file;
mod exporters;
mod importers;
mod interface;
mod otp;
mod path;
mod persistence;
mod reading;
mod utils;

fn init(args: &CotpArgs) -> color_eyre::Result<ReadResult> {
    let path = init_path(args);
    if !path.exists()
        && !matches!(
            args.command,
            None | Some(arguments::CotpSubcommands::Add(_) | arguments::CotpSubcommands::Import(_))
        )
    {
        return Err(eyre!(
            "Vault does not exist. Create it with cotp add or cotp import."
        ));
    }

    match utils::init_app() {
        Ok(first_run) => {
            persistence::initialize(&path)?;
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
    if let Some(arguments::CotpSubcommands::Backup(args)) = &cotp_args.command {
        let path = init_path(&cotp_args);
        return backup::run(args, &path, cotp_args.password_from_stdin).map_err(|e| e.into());
    }
    let (mut database, key, salt) = match init(&cotp_args) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(-1);
        }
    };

    let mut key = zeroize::Zeroizing::new(key);
    persistence::unlock(&key, &salt)?;
    // Complete format upgrades before commands so transactional HOTP starts clean.
    otp::migrations::migrate(&mut database)?;
    if database.is_modified() {
        database.save(&key, &salt)?;
    }
    key.zeroize();

    let mut reowned_database = match args_parser(cotp_args, database) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("An error occurred: {e}");
            persistence::lock();
            std::process::exit(-2)
        }
    };

    let error_code = if reowned_database.is_modified() {
        match persistence::persist(&mut reowned_database) {
            Ok(()) => {
                eprintln!("Modifications have been persisted");
                0
            }
            Err(e) => {
                eprintln!("An error occurred during database overwriting: {e}");
                -1
            }
        }
    } else {
        0
    };
    persistence::lock();
    std::process::exit(error_code)
}

fn dashboard(mut database: OTPDatabase) -> AppResult<OTPDatabase> {
    if !io::stderr().is_terminal() || !io::stdin().is_terminal() {
        if database.elements_ref().is_empty() {
            println!("No codes, type \"cotp -h\" to get help");
            return Ok(database);
        }
        return Err(
            "The dashboard needs an interactive terminal. Use cotp list or cotp list --json."
                .into(),
        );
    }
    let mut app = interface::app::App::new(&mut database);
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal, EventHandler::new(250));
    tui.init()?;
    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Tick => app.tick(false),
            Event::Key(key) => handle_key_events(key, &mut app),
            Event::Mouse(mouse) => handle_mouse_events(mouse, &mut app),
            Event::Paste(mut text) => {
                handle_paste(&text, &mut app);
                text.zeroize();
            }
            Event::Resize => {}
        }
    }
    tui.exit()?;
    Ok(database)
}
