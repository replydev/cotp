use crate::{clipboard, otp::otp_element::OTPDatabase};
use clap::Args;

use super::{SubcommandExecutor, selector::AccountSelector};

#[derive(Args, Default)]
pub struct ExtractArgs {
    #[command(flatten)]
    pub selector: AccountSelector,
    /// Copy the code to the clipboard
    #[arg(short, long = "copy-clipboard", default_value_t = false)]
    pub copy_to_clipboard: bool,
}

impl SubcommandExecutor for ExtractArgs {
    fn run_command(self, mut otp_database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let index = self.selector.select(&otp_database)?;
        let code =
            if otp_database.elements_ref()[index].type_ == crate::otp::otp_type::OTPType::Hotp {
                crate::persistence::consume_hotp(&mut otp_database, index)?
            } else {
                otp_database.elements_ref()[index].get_otp_code()?
            };
        if self.copy_to_clipboard {
            clipboard::copy_string_to_clipboard(&code)?;
            eprintln!("Copied to clipboard");
        }
        println!("{code}");
        Ok(otp_database)
    }
}
