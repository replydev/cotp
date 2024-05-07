use clap::Args;

use crate::otp::otp_element::OTPDatabase;

use super::SubcommandExecutor;

#[derive(Args)]
pub struct ListArgs {
    /// List output format
    #[command(flatten)]
    pub format: Option<ListFormat>,
}

/// Defines the output formats of the list subcommand
#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct ListFormat {
    /// List OTP codes in plain format
    #[arg(short, long)]
    pub plain: bool,

    /// List OTP codes in JSON format
    #[arg(short = 'e', long)]
    pub json: bool,
}

impl Default for ListFormat {
    fn default() -> Self {
        Self {
            plain: true,
            json: false,
        }
    }
}

impl SubcommandExecutor for ListArgs {
    fn run_command(self, otp_database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        todo!()
    }
}
