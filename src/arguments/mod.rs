use crate::otp::otp_element::OTPDatabase;
use crate::{arguments::extract::ExtractArgs, dashboard};
use clap::{Parser, Subcommand};
use color_eyre::eyre::eyre;
use delete::DeleteArgs;
use enum_dispatch::enum_dispatch;

use self::{
    add::AddArgs, edit::EditArgs, export::ExportArgs, import::ImportArgs, list::ListArgs,
    passwd::PasswdArgs,
};

mod add;
mod delete;
mod edit;
mod export;
mod extract;
mod import;
mod list;
mod passwd;

/// Common trait the all the Subcommands must implement to define the command logic
#[enum_dispatch]
pub trait SubcommandExecutor {
    fn run_command(self, otp_database: OTPDatabase) -> color_eyre::Result<OTPDatabase>;
}

/// Main structure defining the Clap argument for the cotp commandline utility
#[derive(Parser)]
#[command(author, version = env!("COTP_VERSION"), about = "Manage one-time passwords in an encrypted vault", long_about = None,
    styles = cli_styles(),
    after_help = "QUICK START
  cotp                                     Open the dashboard
  cotp add --issuer GitHub --label you       Add an account
  cotp list                                Show current codes
  cotp list --json                          Output codes as JSON
  cotp import --help                        Import accounts from another app

DASHBOARD
  a add · e edit · / search · Enter copy · Space QR code · ? help
  q quit · changes are saved only if you confirm

Use cotp <COMMAND> --help for command options and examples."
)]
pub struct CotpArgs {
    #[command(subcommand)]
    command: Option<CotpSubcommands>,
    /// Read the vault password from the first line of standard input
    #[arg(long = "password-stdin", default_value_t = false)]
    pub password_from_stdin: bool,
    /// Path to the encrypted vault file (overrides COTP_DB_PATH)
    #[arg(short = 'd', long = "database-path")]
    pub database_path: Option<String>,
}

/// Define available Subcommands
#[derive(Subcommand)]
#[enum_dispatch(SubcommandExecutor)]
pub enum CotpSubcommands {
    /// Add an account
    Add(AddArgs),
    /// Update an account
    Edit(EditArgs),
    /// Show current one-time passwords
    List(ListArgs),
    /// Delete an account after confirmation
    Delete(DeleteArgs),
    /// Import accounts from a backup
    Import(ImportArgs),
    /// Export accounts to an unencrypted backup
    Export(ExportArgs),
    /// Print the first matching code, optionally copying it to the clipboard
    Extract(ExtractArgs),
    /// Change the vault password
    Passwd(PasswdArgs),
}

pub fn args_parser(matches: CotpArgs, read_result: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
    if let Some(command) = matches.command {
        command.run_command(read_result)
    } else {
        dashboard(read_result).map_err(|e| eyre!("{e}"))
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    CotpArgs::command().debug_assert();
}

fn cli_styles() -> clap::builder::Styles {
    use clap::builder::styling::{AnsiColor, Effects};
    clap::builder::Styles::styled()
        .header(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .usage(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .literal(AnsiColor::Green.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Yellow.on_default())
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
}
