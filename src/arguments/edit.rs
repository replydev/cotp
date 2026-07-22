use clap::{Args, value_parser};
use color_eyre::eyre::eyre;

use crate::otp::{
    otp_algorithm::OTPAlgorithm,
    otp_element::{OTPDatabase, OTPElement, OTPElementBuilder},
};

use super::SubcommandExecutor;

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
    pub algorithm: Option<OTPAlgorithm>,

    /// Code digits
    #[arg(short, long, value_parser=value_parser!(u64).range(1..=10))]
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

impl SubcommandExecutor for EditArgs {
    fn run_command(self, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let secret = self
            .change_secret
            .then(|| rpassword::prompt_password("Insert the secret: "))
            .transpose()?;

        // User provides row number from dashboard which is equal to the array index plus one
        let index = self.index;

        if let Some(real_index) = index.checked_sub(1) {
            if real_index >= database.elements_ref().len() {
                return Err(eyre!("{index} is an invalid index"));
            }

            match database.mut_element(real_index) {
                Some(element) => {
                    let unmodified_element = element.clone();
                    if let Some(v) = self.issuer {
                        element.issuer = v;
                    }
                    if let Some(v) = self.label {
                        element.label = v;
                    }
                    if let Some(v) = self.digits {
                        element.digits = v;
                    }
                    if let Some(v) = self.period {
                        element.period = v;
                    }
                    if let Some(v) = self.algorithm {
                        element.algorithm = v;
                    }
                    if self.counter.is_some() {
                        element.counter = self.counter;
                    }
                    if self.pin.is_some() {
                        element.pin = self.pin;
                    }
                    if let Some(s) = secret {
                        element.secret = validate_secret(element, s)?;
                    }
                    // Only persist (re-encrypt and rewrite the database) if
                    // the edit actually changed something
                    if *element != unmodified_element {
                        database.mark_modified();
                    }
                }
                None => return Err(eyre!("No element found at index {index}")),
            }
            Ok(database)
        } else {
            Err(eyre!("{index} is an invalid index"))
        }
    }
}

/// Run the new secret through the same validation and case normalization that
/// `add` gets via OTPElementBuilder (base32/hex checks depending on the OTP
/// type), instead of persisting the raw string and only failing later at code
/// generation time.
fn validate_secret(element: &OTPElement, secret: String) -> color_eyre::Result<String> {
    let validated = OTPElementBuilder::default()
        .secret(secret)
        .issuer(element.issuer.as_str())
        .label(element.label.as_str())
        .digits(element.digits)
        .type_(element.type_)
        .algorithm(element.algorithm)
        .period(element.period)
        .counter(element.counter)
        .pin(element.pin.clone())
        .build()?;
    Ok(validated.secret.clone())
}
