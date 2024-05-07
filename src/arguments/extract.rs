use crate::otp::otp_element::OTPDatabase;
use crate::{clipboard, otp::otp_element::OTPElement};
use clap::Args;
use color_eyre::eyre::eyre;

use super::SubcommandExecutor;

#[derive(Args)]
pub struct ExtractArgs {
    /// Code Index
    #[arg(short, long, required_unless_present_any = ["issuer", "label"])]
    pub index: Option<usize>,

    /// Code issuer
    #[arg(short = 's', long, required_unless_present_any = ["index", "label"])]
    pub issuer: Option<String>,

    /// Code label
    #[arg(short, long, required_unless_present_any = ["index", "issuer"])]
    pub label: Option<String>,

    /// Copy the code to the clipboard
    #[arg(short, long = "copy-clipboard", default_value_t = false)]
    pub copy_to_clipboard: bool,
}

impl SubcommandExecutor for ExtractArgs {
    fn run_command(self, otp_database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let first_with_filters = otp_database
            .elements
            .iter()
            .enumerate()
            .find(|(index, code)| filter_extract(&self, index, code))
            .map(|(_, code)| code);

        if let Some(otp) = first_with_filters {
            let code = otp.get_otp_code()?;
            println!("{}", code);
            if self.copy_to_clipboard {
                let _ = clipboard::copy_string_to_clipboard(code.as_str())?;
                println!("Copied to clipboard");
            }
            Ok(otp_database)
        } else {
            Err(eyre!("No such code found with these fields"))
        }
    }
}

fn filter_extract(args: &ExtractArgs, index: &usize, code: &OTPElement) -> bool {
    let match_by_index = args.index.map_or(true, |i| i == *index);

    let match_by_issuer = args.issuer.as_ref().map_or(true, |issuer| {
        code.issuer.to_lowercase() == issuer.to_lowercase()
    });

    let match_by_label = args.label.as_ref().map_or(true, |label| {
        code.label.to_lowercase() == label.to_lowercase()
    });

    match_by_index && match_by_issuer && match_by_label
}
