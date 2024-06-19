use clap::{value_parser, Args};
use color_eyre::eyre::eyre;

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPDatabase};

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
    #[arg(short, long, value_parser=value_parser!(u64).range(0..=9))]
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
            .then(|| rpassword::prompt_password("Insert the secret: ").unwrap());

        // User provides row number from dashboard which is equal to the array index plus one
        let index = self.index;

        if let Some(real_index) = index.checked_sub(1) {
            if real_index >= database.elements_ref().len() {
                return Err(eyre!("{index} is an invalid index"));
            }

            match database.mut_element(real_index) {
                Some(element) => {
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
                        element.secret = s;
                    }
                    database.mark_modified();
                }
                None => return Err(eyre!("No element found at index {index}")),
            }
            Ok(database)
        } else {
            Err(eyre!("{index} is an invalid index"))
        }
    }
}
