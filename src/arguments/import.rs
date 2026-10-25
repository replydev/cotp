use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::eyre;

use crate::{
    exporters::otp_uri::OtpUriList,
    importers::{
        aegis::AegisJson, aegis_encrypted::AegisEncryptedDatabase,
        authy_remote_debug::AuthyExportedList, converted::ConvertedJsonList,
        freeotp_plus::FreeOTPPlusJson, google_authenticator::import_from_google_authenticator,
        importer::import_from_path,
    },
    otp::otp_element::{OTPDatabase, OTPElement},
};

use super::SubcommandExecutor;

#[derive(Args)]
#[command(after_help = "EXAMPLES
  cotp import --aegis --path backup.json
  cotp import --google-authenticator --path migration-uris.txt

Choose the format matching your backup. Imported accounts are added to the current vault.
Authy, FreeOTP, and Microsoft Authenticator data must first be converted using the corresponding script in converters/.")]
pub struct ImportArgs {
    #[command(flatten)]
    pub backup_type: BackupType,

    /// Path to the backup file to import
    #[arg(short, long)]
    pub path: PathBuf,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct BackupType {
    /// Import an unencrypted cotp backup
    #[arg(short, long)]
    pub cotp: bool,

    /// Import an andOTP backup
    #[arg(short = 'e', long)]
    pub andotp: bool,

    /// Import an unencrypted Aegis backup
    #[arg(short, long)]
    pub aegis: bool,

    /// Import an encrypted Aegis backup (prompts for its password)
    #[arg(short = 'k', long = "aegis-encrypted")]
    pub aegis_encrypted: bool,

    /// Import a FreeOTP+ backup
    #[arg(short, long = "freeotp-plus")]
    pub freeotp_plus: bool,

    /// Import FreeOTP data converted with converters/freeotp.py
    #[arg(short = 'r', long)]
    pub freeotp: bool,

    /// Import from Google Authenticator: a file containing one or more
    /// `otpauth-migration://` URIs obtained from the "Export accounts" QR codes
    #[arg(short, long = "google-authenticator")]
    pub google_authenticator: bool,

    /// Import Authy data converted with converters/authy.py
    #[arg(short = 't', long)]
    pub authy: bool,

    /// Import an Authy desktop export
    ///
    /// See https://gist.github.com/gboudreau/94bb0c11a6209c82418d01a59d958c93
    #[arg(short = 'u', long = "authy-exported")]
    pub authy_exported: bool,

    /// Import Microsoft Authenticator data converted with converters/mauth.py
    #[arg(short = 'm', long = "microsoft-authenticator")]
    pub microsoft_authenticator: bool,

    /// Import a JSON object with an items array of otpauth:// URIs
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
        } else if backup_type.google_authenticator {
            import_from_google_authenticator(path)
        } else if backup_type.authy || backup_type.microsoft_authenticator || backup_type.freeotp {
            import_from_path::<ConvertedJsonList>(path)
        } else if backup_type.otp_uri {
            import_from_path::<OtpUriList>(path)
        } else {
            return Err(eyre!(
                "Select a backup format. Run cotp import --help for available formats"
            ));
        };

        let elements = result.map_err(|e| eyre!("{e}"))?;

        database.add_all(elements);
        Ok(database)
    }
}
