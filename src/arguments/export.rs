use std::path::PathBuf;

use clap::Args;
use eyre::eyre;

use crate::{
    exporters::{do_export, otp_uri::OtpUriList},
    importers::freeotp_plus::FreeOTPPlusJson,
    otp::otp_element::{OTPDatabase, OTPElement},
};

use super::SubcommandExecutor;

#[derive(Args)]
pub struct ExportArgs {
    /// Export file path
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Export format
    #[command(flatten)]
    pub format: Option<ExportFormat>,
}

#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct ExportFormat {
    /// Export into cotp backup
    #[arg(short, long)]
    pub cotp: bool,

    /// Export into andOTP backup
    #[arg(short = 'e', long)]
    pub andotp: bool,

    /// Export into an OTP URI
    #[arg(short, long = "otp-uri")]
    pub otp_uri: bool,

    /// Export into the `FreeOTP`+ database format
    #[arg(short, long = "freeotp-plus")]
    pub freeotp_plus: bool,
}

/// The export format selected on the command line, derived from the mutually
/// exclusive [`ExportFormat`] flags.
#[derive(Clone, Copy)]
enum ExportKind {
    Cotp,
    Andotp,
    OtpUri,
    FreeOtpPlus,
}

impl ExportFormat {
    /// Maps the mutually exclusive clap flags to the selected export format.
    ///
    /// The clap `ArgGroup` on [`ExportFormat`] (`multiple = false`) combined
    /// with the `Option` flattening in [`ExportArgs`] guarantees exactly one
    /// flag is set whenever this struct is present, so exactly one entry of
    /// the table below is enabled.
    fn kind(&self) -> ExportKind {
        let flag_table = [
            (self.cotp, ExportKind::Cotp),
            (self.andotp, ExportKind::Andotp),
            (self.otp_uri, ExportKind::OtpUri),
            (self.freeotp_plus, ExportKind::FreeOtpPlus),
        ];
        flag_table
            .into_iter()
            .find_map(|(enabled, kind)| enabled.then_some(kind))
            .expect("clap ArgGroup guarantees exactly one export format flag")
    }
}

impl SubcommandExecutor for ExportArgs {
    fn run_command(self, database: OTPDatabase) -> eyre::Result<OTPDatabase> {
        // Exporting to the cotp format when no flag is given keeps the
        // historical default behavior.
        let export_kind = self
            .format
            .as_ref()
            .map_or(ExportKind::Cotp, ExportFormat::kind);
        let exported_path = if self.path.is_dir() {
            self.path.join("exported.cotp")
        } else {
            self.path
        };

        match export_kind {
            ExportKind::Cotp => do_export(&database, exported_path),
            ExportKind::Andotp => {
                let andotp: &Vec<OTPElement> = (&database).into();
                do_export(&andotp, exported_path)
            }
            ExportKind::OtpUri => {
                let otp_uri_list: OtpUriList = (&database).into();
                do_export(&otp_uri_list, exported_path)
            }
            ExportKind::FreeOtpPlus => {
                let freeotp_plus: FreeOTPPlusJson = (&database).try_into()?;
                do_export(&freeotp_plus, exported_path)
            }
        }
        .map(|path| {
            println!(
                "Exported to path: {}",
                path.to_str().unwrap_or("Failed to encode path")
            );
            database
        })
        .map_err(|e| eyre!("An error occurred while exporting database: {e}"))
    }
}
