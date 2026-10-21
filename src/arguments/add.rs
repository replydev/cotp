use clap::{Args, value_parser};
use color_eyre::eyre::Result;

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

    /// Unique account alias
    #[arg(long)]
    pub alias: Option<String>,

    /// Account tag (repeat for multiple tags)
    #[arg(long = "tag", value_parser = crate::otp::organization::parse_tag)]
    pub tags: Vec<String>,
    /// Mark this account as a favorite
    #[arg(long)]
    pub favorite: bool,
    /// Position before accounts without an explicit order (lowest first)
    #[arg(long, allow_hyphen_values = true)]
    pub order: Option<i64>,

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
        value_parser=value_parser!(u64).range(1..=10)
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

    /// Pass the secret through the standard input
    #[arg(long = "secret-stdin", default_value_t = false)]
    take_secret_from_stdin: bool,
}

impl SubcommandExecutor for AddArgs {
    fn run_command(self, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        if let Some(alias) = &self.alias {
            super::selector::validate_alias(&database, alias, None)?;
        }
        let alias = self.alias.clone();
        let tags = self.tags.clone();
        let favorite = self.favorite;
        let order = self.order;
        let mut otp_element = if self.otp_uri {
            let mut otp_uri = if self.take_secret_from_stdin {
                crate::utils::stdin_line()?
            } else {
                crate::utils::prompt_password("Insert the otp uri: ")?
            };
            let result = OTPElement::from_otp_uri(otp_uri.as_str());
            otp_uri.zeroize();
            result?
        } else {
            get_from_args(self)?
        };

        otp_element.alias = alias;
        crate::otp::organization::add_tags(&mut otp_element.tags, tags);
        otp_element.favorite = favorite;
        otp_element.order = order;
        database.add_element(otp_element)?;
        Ok(database)
    }
}

fn get_from_args(matches: AddArgs) -> color_eyre::Result<OTPElement> {
    let secret = if matches.take_secret_from_stdin {
        crate::utils::stdin_line()
    } else {
        crate::utils::prompt_password("Insert the secret: ")
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
