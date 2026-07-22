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
        default_value_if("otp_type", "steam", "5"),
        value_parser=value_parser!(u64).range(1..=10)
    )]
    pub digits: u64,

    /// Code period
    #[arg(short = 'e', long, default_value_t = 30)]
    pub period: u64,

    /// HOTP counter
    #[arg(short, long, required_if_eq("otp_type", "hotp"))]
    pub counter: Option<u64>,

    /// Yandex / MOTP pin
    #[arg(
        short,
        long,
        required_if_eq("otp_type", "yandex"),
        required_if_eq("otp_type", "motp")
    )]
    pub pin: Option<String>,

    /// Pass the secret through the standard input
    #[arg(long = "secret-stdin", default_value_t = false)]
    take_secret_from_stdin: bool,
}

impl SubcommandExecutor for AddArgs {
    fn run_command(self, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let otp_element = if self.otp_uri {
            let mut otp_uri = rpassword::prompt_password("Insert the otp uri: ")?;
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

/// Backstop for the conditional clap rules above: enforce the per-type
/// invariants even if the declarative rules stop firing (e.g. because of an
/// arg id or value-case mismatch, which silently disables them).
fn validate_type_invariants(matches: &AddArgs) -> color_eyre::Result<()> {
    match matches.otp_type {
        OTPType::Hotp if matches.counter.is_none() => {
            Err(eyre::eyre!("--counter is required for HOTP codes"))
        }
        OTPType::Yandex | OTPType::Motp if matches.pin.is_none() => Err(eyre::eyre!(
            "--pin is required for {} codes",
            matches.otp_type
        )),
        _ => Ok(()),
    }
}

fn get_from_args(matches: AddArgs) -> color_eyre::Result<OTPElement> {
    validate_type_invariants(&matches)?;
    let secret = if matches.take_secret_from_stdin {
        if let Some(password) = io::stdin().lock().lines().next() {
            password.map_err(ErrReport::from)
        } else {
            Err(eyre::eyre!("Error during reading from stdin"))
        }
    } else {
        rpassword::prompt_password("Insert the secret: ").map_err(ErrReport::from)
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

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{AddArgs, validate_type_invariants};
    use crate::otp::otp_type::OTPType;

    #[derive(Parser)]
    struct TestParser {
        #[command(flatten)]
        args: AddArgs,
    }

    fn parse(args: &[&str]) -> Result<AddArgs, clap::Error> {
        TestParser::try_parse_from(args).map(|parsed| parsed.args)
    }

    #[test]
    fn hotp_without_counter_is_rejected() {
        let result = parse(&["add", "-l", "label", "-t", "hotp"]);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn hotp_with_counter_is_accepted() {
        let args = parse(&["add", "-l", "label", "-t", "hotp", "-c", "42"]).unwrap();
        assert_eq!(args.counter, Some(42));
    }

    #[test]
    fn yandex_without_pin_is_rejected() {
        let result = parse(&["add", "-l", "label", "-t", "yandex"]);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn motp_without_pin_is_rejected() {
        let result = parse(&["add", "-l", "label", "-t", "motp"]);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn yandex_with_pin_is_accepted() {
        let args = parse(&["add", "-l", "label", "-t", "yandex", "-p", "5678"]).unwrap();
        assert_eq!(args.pin.as_deref(), Some("5678"));
    }

    #[test]
    fn steam_defaults_to_five_digits() {
        let args = parse(&["add", "-l", "label", "-t", "steam"]).unwrap();
        assert_eq!(args.digits, 5);
    }

    #[test]
    fn steam_explicit_digits_are_kept() {
        let args = parse(&["add", "-l", "label", "-t", "steam", "-d", "7"]).unwrap();
        assert_eq!(args.digits, 7);
    }

    #[test]
    fn totp_defaults_to_six_digits() {
        let args = parse(&["add", "-l", "label"]).unwrap();
        assert_eq!(args.digits, 6);
        assert_eq!(args.otp_type, OTPType::Totp);
    }

    #[test]
    fn backstop_rejects_hotp_without_counter() {
        let mut args = parse(&["add", "-l", "label", "-t", "hotp", "-c", "42"]).unwrap();
        // Simulate the clap rule rotting away again
        args.counter = None;
        assert!(validate_type_invariants(&args).is_err());
    }

    #[test]
    fn backstop_rejects_yandex_and_motp_without_pin() {
        for otp_type in ["yandex", "motp"] {
            let mut args = parse(&["add", "-l", "label", "-t", otp_type, "-p", "1234"]).unwrap();
            // Simulate the clap rule rotting away again
            args.pin = None;
            assert!(validate_type_invariants(&args).is_err());
        }
    }

    #[test]
    fn backstop_accepts_valid_combinations() {
        let args = parse(&["add", "-l", "label", "-t", "hotp", "-c", "1"]).unwrap();
        assert!(validate_type_invariants(&args).is_ok());
        let args = parse(&["add", "-l", "label"]).unwrap();
        assert!(validate_type_invariants(&args).is_ok());
    }
}
