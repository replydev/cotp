use super::{CotpArgs, CotpSubcommands, SubcommandExecutor};
use crate::otp::otp_element::OTPDatabase;
use clap::{Args, CommandFactory};
use std::path::PathBuf;

#[derive(Args)]
pub struct CompletionsArgs {
    #[arg(value_enum)]
    shell: clap_complete::Shell,
}

#[derive(Args)]
pub struct ManArgs {
    /// Directory for the main page and all subcommand pages; stdout if omitted
    #[arg(long)]
    directory: Option<PathBuf>,
}

pub fn handle(args: &CotpArgs) -> color_eyre::Result<bool> {
    match &args.command {
        Some(CotpSubcommands::Completions(args)) => {
            clap_complete::generate(
                args.shell,
                &mut CotpArgs::command(),
                "cotp",
                &mut std::io::stdout(),
            );
        }
        Some(CotpSubcommands::Man(args)) => {
            let command = CotpArgs::command();
            if let Some(directory) = &args.directory {
                std::fs::create_dir_all(directory)?;
                clap_mangen::generate_to(command, directory)?;
            } else {
                clap_mangen::Man::new(command).render(&mut std::io::stdout())?;
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

// These variants are handled before vault initialization.
impl SubcommandExecutor for CompletionsArgs {
    fn run_command(self, _: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        Err(color_eyre::eyre::eyre!(
            "Completion generation must run before vault initialization"
        ))
    }
}
impl SubcommandExecutor for ManArgs {
    fn run_command(self, _: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        Err(color_eyre::eyre::eyre!(
            "Manual generation must run before vault initialization"
        ))
    }
}
