use std::io::{self, BufRead, Write};

#[cfg(unix)]
use std::fs::File;
#[cfg(windows)]
use std::fs::OpenOptions;

use clap::Args;
use color_eyre::eyre::eyre;

use crate::otp::otp_element::OTPDatabase;

use super::SubcommandExecutor;

#[derive(Args)]
#[command(after_help = "EXAMPLES
  cotp delete --index 1
  cotp delete --issuer GitHub --label you@example.com

Selects the first matching account and asks for confirmation before deleting it.")]
pub struct DeleteArgs {
    /// Account index from cotp list (starts at 1)
    #[arg(short, long, required_unless_present_any=["issuer", "label"])]
    pub index: Option<usize>,

    /// Match the first account whose service name contains this text
    #[arg(short = 's', long, required_unless_present_any=["index", "label"])]
    pub issuer: Option<String>,

    /// Match the first account whose label contains this text
    #[arg(short, long, required_unless_present_any=["index","issuer"])]
    pub label: Option<String>,
}

impl SubcommandExecutor for DeleteArgs {
    fn run_command(self, mut otp_database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        if otp_database.elements_ref().is_empty() {
            return Err(eyre!("The vault has no accounts to delete"));
        }

        let index_to_delete = self
            .index
            .and_then(|i| i.checked_sub(1))
            // Match by issues or label if index filter is missing
            .or_else(|| get_first_matching_element(&otp_database, &self))
            .ok_or(eyre!(
                "No account matches the supplied index, issuer, or label"
            ))?;

        if let Some(element) = otp_database.elements_ref().get(index_to_delete) {
            print!(
                "Delete account #{} ({}, {})? [y/N]: ",
                index_to_delete + 1,
                element.issuer,
                element.label
            );
            io::stdout().flush()?;

            let output = read_confirmation_line()?;

            if output.trim().eq_ignore_ascii_case("y") {
                otp_database.delete_element(index_to_delete);
                Ok(otp_database)
            } else {
                Err(eyre!("Deletion canceled"))
            }
        } else {
            Err(eyre!("No account found at index {}", index_to_delete + 1))
        }
    }
}

fn read_confirmation_line() -> color_eyre::Result<String> {
    let mut output = String::with_capacity(1);

    if io::stdin().read_line(&mut output)? > 0 {
        return Ok(output);
    }

    #[cfg(unix)]
    {
        output.clear();
        let mut tty = io::BufReader::new(File::open("/dev/tty")?);
        tty.read_line(&mut output)?;
        return Ok(output);
    }

    #[cfg(windows)]
    {
        output.clear();
        let mut tty = io::BufReader::new(OpenOptions::new().read(true).open("CONIN$")?);
        tty.read_line(&mut output)?;
        return Ok(output);
    }

    #[allow(unreachable_code)]
    Err(eyre!(
        "Could not read confirmation from standard input or the terminal"
    ))
}

fn get_first_matching_element(
    otp_database: &OTPDatabase,
    delete_args: &DeleteArgs,
) -> Option<usize> {
    otp_database
        .elements_ref()
        .iter()
        .enumerate()
        .find(|(_, element)| {
            element
                .issuer
                .contains(delete_args.issuer.as_deref().unwrap_or_default())
                && element
                    .label
                    .contains(delete_args.label.as_deref().unwrap_or_default())
        })
        .map(|(index, _)| index)
}
