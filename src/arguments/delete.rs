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
    /// Code Index
    #[arg(short, long, required_unless_present_any=["issuer", "label"])]
    pub index: Option<usize>,

    /// Issuer of the first matching code that will be deleted
    #[arg(short = 's', long, required_unless_present_any=["index", "label"])]
    pub issuer: Option<String>,

    /// Label of the first matching code that will be deleted
    #[arg(short, long, required_unless_present_any=["index","issuer"])]
    pub label: Option<String>,
}

impl SubcommandExecutor for DeleteArgs {
    fn run_command(self, mut otp_database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let index_to_delete = self
            .index
            .and_then(|i| i.checked_sub(1))
            .or_else(|| get_first_matching_element(&otp_database, &self))
            .ok_or(eyre!("No code has been found using the given arguments"))?;

        let element = otp_database.elements_ref().get(index_to_delete).unwrap();
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
