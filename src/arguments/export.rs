use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::eyre;

use crate::{
    exporters::{do_export, otp_uri::OtpUriList},
    importers::freeotp_plus::FreeOTPPlusJson,
    otp::otp_element::{OTPDatabase, OTPElement},
};

use super::SubcommandExecutor;

#[derive(Args)]
#[command(after_help = "EXAMPLES
  cotp export --path backup.json
  cotp export --otp-uri --path accounts.json

Exports contain account secrets and are not encrypted. Store them securely.
If --path is a directory, the output file is named exported.cotp regardless of format.")]
pub struct ExportArgs {
    /// Output file or directory (directories use the filename exported.cotp)
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Export format
    #[command(flatten)]
    pub format: Option<ExportFormat>,
}

#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct ExportFormat {
    /// Export an unencrypted cotp backup (default)
    #[arg(short, long)]
    pub cotp: bool,

    /// Export an andOTP backup
    #[arg(short = 'e', long)]
    pub andotp: bool,

    /// Export a JSON object with an items array of otpauth:// URIs
    #[arg(short, long = "otp-uri")]
    pub otp_uri: bool,

    /// Export a FreeOTP+ backup
    #[arg(short, long = "freeotp-plus")]
    pub freeotp_plus: bool,
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self {
            cotp: true,
            andotp: false,
            otp_uri: false,
            freeotp_plus: false,
        }
    }
}

impl SubcommandExecutor for ExportArgs {
    fn run_command(self, database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let export_format = self.format.unwrap_or_default();
        let exported_path = if self.path.is_dir() {
            self.path.join("exported.cotp")
        } else {
            self.path
        };

        if export_format.cotp {
            do_export(&database, exported_path)
        } else if export_format.andotp {
            let andotp: &Vec<OTPElement> = (&database).into();
            do_export(&andotp, exported_path)
        } else if export_format.otp_uri {
            let otp_uri_list: OtpUriList = (&database).into();
            do_export(&otp_uri_list, exported_path)
        } else if export_format.freeotp_plus {
            let freeotp_plus: FreeOTPPlusJson = (&database).try_into()?;
            do_export(&freeotp_plus, exported_path)
        } else {
            unreachable!("Unreachable code");
        }
        .map(|path| {
            println!("Accounts exported to {}", path.display());
            database
        })
        .map_err(|e| eyre!("Could not export accounts: {e}"))
    }
}
