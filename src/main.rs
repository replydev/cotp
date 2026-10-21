#![forbid(unsafe_code)]
use arguments::{CotpArgs, args_parser};
use clap::Parser;
use color_eyre::eyre::eyre;
use interface::app::AppResult;
use interface::event::{Event, EventHandler};
use interface::handlers::{handle_key_events, handle_mouse_events, handle_paste};
use interface::ui::Tui;
use otp::otp_element::OTPDatabase;
use path::init_path;
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use reading::ReadResult;
use std::io::{self, IsTerminal};
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
    let explicit_init = matches!(args.command, Some(arguments::CotpSubcommands::Init(_)));
    if explicit_init && path.exists() {
        return Err(eyre!("Vault already exists; init never overwrites a vault"));
    }
    if !path.exists()
        && !matches!(
            args.command,
            None | Some(
                arguments::CotpSubcommands::Init(_)
                    | arguments::CotpSubcommands::Add(_)
                    | arguments::CotpSubcommands::Import(_)
            )
        )
    {
        return Err(eyre!(
            "Vault does not exist. Create it with cotp init, add, or import."
        ));
    }
    let first_run = utils::init_app().map_err(|()| eyre!("Could not create vault directory"))?;
    persistence::initialize(&path)?;
    let explicit_password = if args.password_from_stdin {
        Some(utils::stdin_line()?)
    } else {
        args.password_fd.map(utils::password_from_fd).transpose()?
    };
    let password = zeroize::Zeroizing::new(match explicit_password {
        Some(password) => password,
        None if first_run => utils::verified_password("Choose a password: ", 8)?,
        None => utils::password("Password: ", 0)?,
    });
    if first_run {
        if password.chars().count() < 8 {
            return Err(eyre!(
                "New vault password must contain at least 8 characters"
            ));
        }
        let mut database = OTPDatabase::default();
        let (key, salt) = database.save_with_pw(&password)?;
        Ok((database, key, salt.to_vec()))
    } else {
        reading::read_from_file(&password)
    }
}

fn run(cotp_args: CotpArgs) -> Result<(), (u8, color_eyre::eyre::Report)> {
    utils::set_no_input(cotp_args.no_input);
    if let Some(arguments::CotpSubcommands::Backup(args)) = &cotp_args.command {
        let path = init_path(&cotp_args);
        return backup::run(
            args,
            &path,
            cotp_args.password_from_stdin,
            cotp_args.password_fd,
        )
        .map_err(|error| (1, error));
    }
    let (mut database, key, salt) = init(&cotp_args).map_err(|error| (1, error))?;
    let mut key = zeroize::Zeroizing::new(key);
    persistence::unlock(&key, &salt).map_err(|error| (1, error))?;
    if database.is_modified() {
        database.save(&key, &salt).map_err(|error| (1, error))?;
    }
    key.zeroize();
    let mut database = args_parser(cotp_args, database).map_err(|error| (3, error))?;
    if database.is_modified() {
        persistence::persist(&mut database).map_err(|error| {
            (
                1,
                eyre!("An error occurred during database overwriting: {error}"),
            )
        })?;
        eprintln!("Modifications have been persisted");
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    if let Err(error) = color_eyre::install() {
        eprintln!("{error}");
        return std::process::ExitCode::FAILURE;
    }
    let result = run(CotpArgs::parse());
    persistence::lock();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err((code, error)) => {
            eprintln!("{error}");
            std::process::ExitCode::from(code)
        }
    }
}

fn dashboard(mut database: OTPDatabase) -> AppResult<OTPDatabase> {
    utils::ensure_interactive()?;
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
