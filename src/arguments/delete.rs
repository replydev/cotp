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
pub struct DeleteArgs {
    #[command(flatten)]
    pub selector: super::selector::AccountSelector,
}

impl SubcommandExecutor for DeleteArgs {
    fn run_command(self, mut otp_database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        if otp_database.elements_ref().is_empty() {
            return Err(eyre!("There are no elements to delete"));
        }

        let index_to_delete = self.selector.select(&otp_database)?;

        if let Some(element) = otp_database.elements_ref().get(index_to_delete) {
            print!(
                "Are you sure you want to delete the {}th code ({}, {}) [Y,N]: ",
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
                Err(eyre!("Operation interrupt by the user"))
            }
        } else {
            Err(eyre!("Missing {}th code to delete", index_to_delete + 1))
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
        "Unable to read confirmation answer from standard input or terminal"
    ))
}
