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
    #[arg(short = 't', long = "type", default_value = "totp")]
    pub otp_type: OTPType,

    /// Code issuer
    #[arg(short, long, required_unless_present = "otp_uri")]
    pub issuer: Option<String>,

    /// Code label
    #[arg(short, long, default_value = "")]
    pub label: String,

    /// OTP Algorithm
    #[arg(short, long, value_enum, default_value_t = OTPAlgorithm::Sha1)]
    pub algorithm: OTPAlgorithm,

    /// Code digits
    #[arg(
        short,
        long,
        default_value_t = 6,
        default_value_if("type", "STEAM", "5")
    )]
    pub digits: u64,

    /// Code period
    #[arg(short = 'e', long, default_value_t = 30)]
    pub period: u64,

    /// HOTP counter
    #[arg(short, long, required_if_eq("otp_type", "HOTP"))]
    pub counter: Option<u64>,

    /// Yandex / MOTP pin
    #[arg(
        short,
        long,
        required_if_eq("otp_type", "YANDEX"),
        required_if_eq("otp_type", "MOTP")
    )]
    pub pin: Option<String>,
}

#[derive(Args)]
pub struct EditArgs {
    /// Code Index
    #[arg(short, long)]
    pub index: usize,

    /// Code issuer
    #[arg(short = 's', long)]
    pub issuer: Option<String>,

    /// Code label
    #[arg(short, long)]
    pub label: Option<String>,

    /// OTP algorithm
    #[arg(short, long, value_enum)]
    pub mode: Option<OTPAlgorithm>,

    /// Code digits
    #[arg(short, long)]
    pub digits: Option<u64>,

    /// Code period
    #[arg(short = 'e', long)]
    pub period: Option<u64>,

    /// HOTP counter
    #[arg(short, long)]
    pub counter: Option<u64>,

    /// Yandex / MOTP pin
    #[arg(short, long)]
    pub pin: Option<String>,

    /// Change code secret
    #[arg(short = 'k', long = "change-secret")]
    pub change_secret: bool,
}

#[derive(Args)]
pub struct ImportArgs {
    #[command(flatten)]
    pub backup_type: BackupType,

    /// Backup file path
    #[arg(short, long)]
    pub path: PathBuf,
}

#[derive(Args)]
pub struct ExportArgs {
    /// Export file path
    #[arg(short, long, default_value = ".")]
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

    /// Import from FreeOTP+ backup
    #[arg(short, long = "freeotp-plus")]
    pub freeotp_plus: bool,

    /// Import from FreeOTP backup
    #[arg(short = 'r', long)]
    pub freeotp: bool,

    /// Import from Google Authenticator backup
    #[arg(short, long = "google-authenticator")]
    pub google_authenticator: bool,

    /// Import from Authy backup
    #[arg(short = 't', long)]
    pub authy: bool,

    /// Import from Authy Database exported following this guide https://gist.github.com/gboudreau/94bb0c11a6209c82418d01a59d958c93
    #[arg(short = 'u', long = "authy-exported")]
    pub authy_exported: bool,

    /// Import from Microsoft Authenticator
    #[arg(short = 'm', long = "microsoft-authenticator")]
    pub microsoft_authenticator: bool,
}

pub fn args_parser(
    matches: CotpArgs,
    database: &mut OTPDatabase,
) -> Option<Result<String, String>> {
    match matches.command {
        Some(CotpSubcommands::Add(args)) => Some(argument_functions::add(args, database)),
        Some(CotpSubcommands::Edit(args)) => Some(argument_functions::edit(args, database)),
        Some(CotpSubcommands::Import(args)) => Some(argument_functions::import(args, database)),
        Some(CotpSubcommands::Export(args)) => Some(argument_functions::export(args, database)),
        Some(CotpSubcommands::Passwd) => Some(argument_functions::change_password(database)),
        None => None,
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    CotpArgs::command().debug_assert()
}
