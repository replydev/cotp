use clap::{Args, value_parser};
use color_eyre::eyre::{ErrReport, Result};

use zeroize::Zeroize;

use crate::otp::{
    from_otp_uri::FromOtpUri,
    otp_algorithm::OTPAlgorithm,
    otp_element::{OTPDatabase, OTPElement, OTPElementBuilder},
    otp_type::OTPType,
};

use super::SubcommandExecutor;

#[derive(Args)]
pub struct AddArgs {
    /// Add OTP code via an OTP URI
    #[arg(short = 'u', long = "otpuri", required_unless_present = "label")]
    pub otp_uri: bool,

    /// Specify the OTP code type
    #[arg(short = 't', long = "type", default_value = "totp")]
    pub otp_type: OTPType,

    /// Code issuer
    #[arg(short, long, default_value = "")]
    pub issuer: String,

    /// Code label
    #[arg(short, long, required_unless_present = "otp_uri")]
    pub label: Option<String>,

    /// OTP Algorithm
    #[arg(short, long, value_enum, default_value_t = OTPAlgorithm::Sha1)]
    pub algorithm: OTPAlgorithm,

    /// Code digits
    #[arg(
        short,
        long,
        default_value_t = 6,
        default_value_if("type", "STEAM", "5"),
        value_parser=value_parser!(u64).range(0..=9)
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

impl SubcommandExecutor for AddArgs {
    fn run_command(self, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let otp_element = if self.otp_uri {
            let mut otp_uri = rpassword::prompt_password("Insert the otp uri: ").unwrap();
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
    let secret = rpassword::prompt_password("Insert the secret: ").map_err(ErrReport::from)?;
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
