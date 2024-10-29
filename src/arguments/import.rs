use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::eyre;

use crate::{
    exporters::otp_uri::OtpUriList,
    importers::{
        aegis::AegisJson, aegis_encrypted::AegisEncryptedDatabase,
        authy_remote_debug::AuthyExportedList, converted::ConvertedJsonList,
        freeotp_plus::FreeOTPPlusJson, importer::import_from_path,
    },
    otp::otp_element::{OTPDatabase, OTPElement},
};

use super::SubcommandExecutor;

#[derive(Args)]
pub struct ImportArgs {
    #[command(flatten)]
    pub backup_type: BackupType,

    /// Backup file path
    #[arg(short, long)]
    pub path: PathBuf,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct BackupType {
    /// Import from cotp backup
    #[arg(short, long)]
    pub cotp: bool,

    /// Import from andOTP backup
    #[arg(short = 'e', long)]
    pub andotp: bool,

    /// Import from Aegis backup
    #[arg(short, long)]
    pub aegis: bool,

    /// Import from Aegis Encrypted backup
    #[arg(short = 'k', long = "aegis-encrypted")]
    pub aegis_encrypted: bool,

    /// Import from `FreeOTP`+ backup
    #[arg(short, long = "freeotp-plus")]
    pub freeotp_plus: bool,

    /// Import from `FreeOTP` backup
    #[arg(short = 'r', long)]
    pub freeotp: bool,

    /// Import from Google Authenticator backup
    #[arg(short, long = "google-authenticator")]
    pub google_authenticator: bool,

    /// Import from Authy backup
    #[arg(short = 't', long)]
    pub authy: bool,

    /// Import from Authy Database exported following this guide <https://gist.github.com/gboudreau/94bb0c11a6209c82418d01a59d958c93>
    #[arg(short = 'u', long = "authy-exported")]
    pub authy_exported: bool,

    /// Import from Microsoft Authenticator
    #[arg(short = 'm', long = "microsoft-authenticator")]
    pub microsoft_authenticator: bool,

    /// Import from OTP Uri batch
    #[arg(short, long = "otp-uri")]
    pub otp_uri: bool,
}

impl SubcommandExecutor for ImportArgs {
    fn run_command(self, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let path = self.path;

        let backup_type = self.backup_type;

        let result = if backup_type.cotp {
            import_from_path::<OTPDatabase>(path)
        } else if backup_type.andotp {
            import_from_path::<Vec<OTPElement>>(path)
        } else if backup_type.aegis {
            import_from_path::<AegisJson>(path)
        } else if backup_type.aegis_encrypted {
            import_from_path::<AegisEncryptedDatabase>(path)
        } else if backup_type.freeotp_plus {
            import_from_path::<FreeOTPPlusJson>(path)
        } else if backup_type.authy_exported {
            import_from_path::<AuthyExportedList>(path)
        } else if backup_type.google_authenticator
            || backup_type.authy
            || backup_type.microsoft_authenticator
            || backup_type.freeotp
        {
            import_from_path::<ConvertedJsonList>(path)
        } else if backup_type.otp_uri {
            import_from_path::<OtpUriList>(path)
        } else {
            return Err(eyre!("Invalid arguments provided"));
        };

        let elements = result.map_err(|e| eyre!("{e}"))?;

        database.add_all(elements);
        Ok(database)
    }
}
