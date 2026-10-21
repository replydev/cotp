use super::SubcommandExecutor;
use crate::{
    importers::{
        aegis_encrypted::AegisEncryptedDatabase,
        enrollment::{self, DuplicatePolicy, ImportReport},
    },
    otp::otp_element::OTPDatabase,
};
use clap::Args;
use color_eyre::eyre::{Result, eyre};
use std::{
    io::{self, IsTerminal, Read, Write},
    path::PathBuf,
};

#[derive(Args)]
pub struct ImportArgs {
    #[command(flatten)]
    pub backup_type: BackupType,
    /// Backup, QR image, or URI text file; use - to read standard input
    #[arg(value_name = "INPUT", conflicts_with = "path")]
    pub input: Option<PathBuf>,
    /// Backup file path (legacy alias for positional INPUT)
    #[arg(short, long, required_unless_present = "input")]
    pub path: Option<PathBuf>,
    /// Preview changes without modifying the vault
    #[arg(long)]
    pub dry_run: bool,
    /// How to handle accounts with the same secret and OTP parameters
    #[arg(long, value_enum, default_value = "skip")]
    pub duplicates: DuplicatePolicy,
    /// Import valid entries even when other entries are invalid
    #[arg(long)]
    pub skip_invalid: bool,
    /// Accept the displayed preview without confirmation
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Args)]
#[group(required = false, multiple = false)]
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

    /// Import from Google Authenticator: a file containing one or more
    /// `otpauth-migration://` URIs obtained from the "Export accounts" QR codes
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

impl BackupType {
    fn selected(&self) -> Option<&'static str> {
        [
            (self.cotp, "cotp"),
            (self.andotp, "andotp"),
            (self.aegis, "aegis"),
            (self.aegis_encrypted, "aegis-encrypted"),
            (self.freeotp_plus, "freeotp-plus"),
            (self.authy_exported, "authy-exported"),
            (self.google_authenticator, "google-authenticator"),
            (
                self.authy || self.microsoft_authenticator || self.freeotp,
                "converted",
            ),
            (self.otp_uri, "otp-uri"),
        ]
        .into_iter()
        .find_map(|(selected, name)| selected.then_some(name))
    }
}

impl SubcommandExecutor for ImportArgs {
    fn run_command(self, mut database: OTPDatabase) -> Result<OTPDatabase> {
        let path = self
            .input
            .or(self.path)
            .ok_or_else(|| eyre!("Import input is required"))?;
        let mut bytes = zeroize::Zeroizing::new(Vec::new());
        if path.as_os_str() == "-" {
            io::stdin()
                .take(32 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)?;
        } else {
            std::fs::File::open(&path)?
                .take(32 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)?;
        }
        if bytes.len() > 32 * 1024 * 1024 {
            return Err(eyre!("Import input exceeds 32 MiB"));
        }
        let report = if image::guess_format(&bytes).is_ok() {
            enrollment::decode_qr(&bytes)?
        } else {
            let content = std::str::from_utf8(&bytes)
                .map_err(|_| eyre!("Input is not UTF-8 text or a supported QR image"))?
                .trim();
            if content.starts_with("otpauth") || self.backup_type.google_authenticator {
                enrollment::parse_uris(content)
            } else {
                let value: serde_json::Value = serde_json::from_str(content).map_err(|_| {
                    eyre!("Invalid JSON; expected a backup, OTP URI text, or QR image")
                })?;
                let format = match self.backup_type.selected() {
                    Some(format) => format,
                    None => enrollment::detect_format(&value)?,
                };
                if format == "aegis-encrypted" {
                    let mut report = ImportReport::default();
                    report.record(1, enrollment::convert::<AegisEncryptedDatabase>(value));
                    report
                } else {
                    enrollment::parse_json(value, format)?
                }
            }
        };
        for error in &report.errors {
            eprintln!("{error}");
        }
        if !report.errors.is_empty() && !self.skip_invalid {
            return Err(eyre!(
                "Import rejected: {} invalid entries; use --skip-invalid to accept valid entries",
                report.errors.len()
            ));
        }
        if report.elements.is_empty() {
            return Err(eyre!("No valid accounts found in import"));
        }
        for element in &report.elements {
            eprintln!(
                "{} / {} ({}, {} digits)",
                element.issuer.escape_default(),
                element.label.escape_default(),
                element.type_,
                element.digits
            );
        }
        eprintln!(
            "{} valid accounts; duplicate policy: {:?}{}",
            report.elements.len(),
            self.duplicates,
            if self.dry_run { " (dry run)" } else { "" }
        );
        if self.dry_run {
            return Ok(database);
        }
        if !self.yes && io::stdin().is_terminal() {
            eprint!("Apply import? [y/N] ");
            io::stderr().flush()?;
            let mut answer = String::new();
            io::stdin().read_line(&mut answer)?;
            if !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
                return Err(eyre!("Import cancelled"));
            }
        }
        let (added, replaced, skipped) =
            enrollment::apply(&mut database, report.elements, self.duplicates)?;
        eprintln!(
            "Import complete: {added} added, {replaced} replaced, {skipped} duplicates skipped"
        );
        Ok(database)
    }
}
