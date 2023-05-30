use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::{
    argument_functions,
    otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPDatabase, otp_type::OTPType},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CotpArgs {
    #[command(subcommand)]
    command: Option<CotpSubcommands>,
}

#[derive(Subcommand)]
enum CotpSubcommands {
    /// Add new OTP code
    Add(AddArgs),
    /// Edit an existing OTP Code
    Edit(EditArgs),
    /// Remove an OTP Code
    Remove(RemoveArgs),
    /// Import codes from other apps
    Import(ImportArgs),
    /// Export cotp database
    Export(ExportArgs),
    /// Change database password
    Passwd,
}

#[derive(Args)]
pub struct AddArgs {
    /// Add OTP code via an OTP URI
    #[arg(short = 'u', long = "otpuri", required_unless_present = "issuer")]
    pub otp_uri: bool,

    /// Specify the OTP code type
    #[arg(short = 't', long = "type", default_value_t = OTPType::Totp)]
    pub otp_type: OTPType,

    /// Code issuer
    #[arg(short, long, required_unless_present = "otp_uri")]
    pub issuer: String,

    /// Code label
    #[arg(short, long, default_value = "")]
    pub label: String,

    /// OTP Algorithm
    #[arg(short, long, value_enum, default_value_t = OTPAlgorithm::Sha1)]
    pub mode: OTPAlgorithm,

    /// Code digits
    #[arg(
        short,
        long,
        default_value_t = 6,
        default_value_if("type", "STEAM", "5")
    )]
    pub digits: u8,

    /// Code period
    #[arg(short, long, default_value_t = 30)]
    pub period: u64,

    /// HOTP counter
    #[arg(short, long, required_if_eq("type", "HOTP"))]
    pub counter: Option<u64>,

    /// Yandex / MOTP pin
    #[arg(
        short,
        long,
        required_if_eq("type", "YANDEX"),
        required_if_eq("type", "MOTP")
    )]
    pub pin: Option<String>,
}

#[derive(Args)]
pub struct EditArgs {
    /// Code Index
    #[arg(short, long, required = true)]
    index: u64,

    /// Code issuer
    #[arg(short = 's', long)]
    issuer: Option<String>,

    /// Code label
    #[arg(short, long)]
    label: Option<String>,

    /// OTP algorithm
    #[arg(short, long, value_enum)]
    mode: Option<OTPAlgorithm>,

    /// Code digits
    #[arg(short, long)]
    digits: Option<u8>,

    /// Code period
    #[arg(short, long)]
    period: Option<u64>,

    /// HOTP counter
    #[arg(short, long)]
    counter: Option<u64>,

    /// Yandex / MOTP pin
    #[arg(short, long)]
    pin: Option<String>,

    /// Change code secret
    #[arg(short = 'k', long = "change-secret")]
    change_secret: bool,
}

#[derive(Args)]
pub struct RemoveArgs {
    /// Code Index
    #[arg(short, long, required = true)]
    index: u64,
}

#[derive(Args)]
pub struct ImportArgs {
    #[command(flatten)]
    backup_type: BackupType,

    /// Backup file path
    #[arg(short, long, required = true)]
    path: PathBuf,
}

#[derive(Args)]
pub struct ExportArgs {
    /// Export file path
    #[arg(short, long, default_value = ".")]
    path: PathBuf,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct BackupType {
    /// Import from cotp backup
    #[arg(short, long)]
    cotp: bool,

    /// Import from andOTP backup
    #[arg(short = 'e', long)]
    andotp: bool,

    /// Import from Aegis backup
    #[arg(short, long)]
    aegis: bool,

    /// Import from Aegis Encrypted backup
    #[arg(short = 'k', long = "aegis-encrypted")]
    aegis_encrypted: bool,

    /// Import from FreeOTP+ backup
    #[arg(short, long = "freeotp-plus")]
    freeotp_plus: bool,

    /// Import from FreeOTP backup
    #[arg(short = 'r', long)]
    freeotp: bool,

    /// Import from Google Authenticator backup
    #[arg(short, long = "google-authenticator")]
    google_authenticator: bool,

    /// Import from Authy backup
    #[arg(short = 't', long)]
    authy: bool,

    /// Import from Authy Database exported following this guide https://gist.github.com/gboudreau/94bb0c11a6209c82418d01a59d958c93
    #[arg(short = 'u', long = "authy-exported")]
    authy_exported: bool,

    /// Import from Microsoft Authenticator
    #[arg(short = 'm', long = "microsoft-authenticator")]
    microsoft_authenticator: bool,
}

pub fn args_parser(
    matches: CotpArgs,
    database: &mut OTPDatabase,
) -> Option<Result<String, String>> {
    match matches.command {
        Some(CotpSubcommands::Add(args)) => Some(argument_functions::add(args, database)),
        Some(("edit", edit_matches)) => Some(argument_functions::edit(edit_matches, database)),
        Some(("import", import_matches)) => {
            Some(argument_functions::import(import_matches, database))
        }
        Some(("export", export_matches)) => {
            Some(argument_functions::export(export_matches, database))
        }
        Some(("passwd", _)) => Some(argument_functions::change_password(database)),
        Some((_, _)) => Some(Err(String::from("Invalid args"))),
        None => None,
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    CotpArgs::command().debug_assert()
}
