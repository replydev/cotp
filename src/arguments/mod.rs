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
#[command(author, version = env!("COTP_VERSION"), about, long_about = None)]
pub struct CotpArgs {
    #[command(subcommand)]
    command: Option<CotpSubcommands>,
    /// Fetch the password from standard input
    #[arg(long = "password-stdin", default_value_t = false)]
    pub password_from_stdin: bool,
    /// Set the database path
    #[arg(short = 'd', long = "database-path")]
    pub database_path: Option<String>,
}

/// Define available Subcommands
#[derive(Subcommand)]
#[enum_dispatch(SubcommandExecutor)]
pub enum CotpSubcommands {
    /// Add new OTP code
    Add(AddArgs),
    /// Edit an existing OTP Code
    Edit(EditArgs),
    /// List codes
    List(ListArgs),
    /// Delete codes
    Delete(DeleteArgs),
    /// Import codes from other apps
    Import(ImportArgs),
    /// Export cotp database
    Export(ExportArgs),
    /// Copies the selected code into the clipboard, supports glob matching
    Extract(ExtractArgs),
    /// Change database password
    Passwd(PasswdArgs),
}

pub fn args_parser(matches: CotpArgs, read_result: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
    if let Some(command) = matches.command {
        command.run_command(read_result)
    } else {
        dashboard(read_result).map_err(|e| eyre!("An error occurred: {e}"))
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    CotpArgs::command().debug_assert();
}
