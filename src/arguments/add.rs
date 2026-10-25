use std::io::{self, BufRead};

use clap::{Args, value_parser};
use color_eyre::eyre::{self, ErrReport, Result};

use zeroize::Zeroize;

use crate::otp::{
    from_otp_uri::FromOtpUri,
    otp_algorithm::OTPAlgorithm,
    otp_element::{OTPDatabase, OTPElement, OTPElementBuilder},
    otp_type::OTPType,
};

use super::SubcommandExecutor;

#[derive(Args)]
#[command(after_help = "EXAMPLES
  cotp add --issuer GitHub --label you@example.com
  cotp add --otpuri

You will be prompted for the secret key or OTP URI. Input is hidden.")]
pub struct AddArgs {
    /// Prompt for an otpauth:// URI instead of individual account fields
    #[arg(short = 'u', long = "otpuri", required_unless_present = "label")]
    pub otp_uri: bool,

    /// Account type
    #[arg(short = 't', long = "type", default_value = "totp")]
    pub otp_type: OTPType,

    /// Service name, such as GitHub
    #[arg(short, long, default_value = "")]
    pub issuer: String,

    /// Account name, such as an email address or username
    #[arg(short, long, required_unless_present = "otp_uri")]
    pub label: Option<String>,

    /// Hash algorithm used to generate codes
    #[arg(short, long, value_enum, default_value_t = OTPAlgorithm::Sha1)]
    pub algorithm: OTPAlgorithm,

    /// Number of characters in each code (1-10)
    #[arg(
        short,
        long,
        default_value_t = 6,
        default_value_if("type", "STEAM", "5"),
        value_parser=value_parser!(u64).range(1..=10)
    )]
    pub digits: u64,

    /// Code refresh period in seconds
    #[arg(short = 'e', long, default_value_t = 30)]
    pub period: u64,

    /// Initial counter for HOTP accounts
    #[arg(short, long, required_if_eq("otp_type", "HOTP"))]
    pub counter: Option<u64>,

    /// PIN for Yandex or MOTP accounts
    #[arg(
        short,
        long,
        required_if_eq("otp_type", "YANDEX"),
        required_if_eq("otp_type", "MOTP")
    )]
    pub pin: Option<String>,

    /// Read the secret from standard input (after the password with --password-stdin)
    #[arg(long = "secret-stdin", default_value_t = false)]
    take_secret_from_stdin: bool,
}

impl SubcommandExecutor for AddArgs {
    fn run_command(self, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let otp_element = if self.otp_uri {
            let mut otp_uri = rpassword::prompt_password("OTP URI (otpauth://): ").unwrap();
            let result = OTPElement::from_otp_uri(otp_uri.as_str());
            otp_uri.zeroize();
            result?
        } else {
            get_from_args(self)?
        };

        database.add_element(otp_element);
        Ok(database)
    }
}

fn get_from_args(matches: AddArgs) -> color_eyre::Result<OTPElement> {
    let secret = if matches.take_secret_from_stdin {
        if let Some(password) = io::stdin().lock().lines().next() {
            password.map_err(ErrReport::from)
        } else {
            Err(eyre::eyre!("No secret key received on standard input"))
        }
    } else {
        rpassword::prompt_password("Secret key: ").map_err(ErrReport::from)
    }?;
    map_args_to_code(secret, matches)
}

fn map_args_to_code(secret: String, matches: AddArgs) -> Result<OTPElement> {
    OTPElementBuilder::default()
        .secret(secret)
        .issuer(matches.issuer)
        .label(matches.label.unwrap())
        .digits(matches.digits)
        .type_(matches.otp_type)
        .algorithm(matches.algorithm)
        .period(matches.period)
        .counter(matches.counter)
        .pin(matches.pin)
        .build()
}
