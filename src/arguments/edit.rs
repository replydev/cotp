use clap::{Args, value_parser};
use color_eyre::eyre::eyre;

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPDatabase};

use super::SubcommandExecutor;

#[derive(Args)]
#[command(after_help = "EXAMPLE
  cotp edit --index 1 --label you@example.com

Use cotp list to find the account index. Omitted fields keep their current values.")]
pub struct EditArgs {
    /// Account index from cotp list (starts at 1)
    #[arg(short, long)]
    pub index: usize,

    /// New service name
    #[arg(short = 's', long)]
    pub issuer: Option<String>,

    /// New account name
    #[arg(short, long)]
    pub label: Option<String>,

    /// New hash algorithm
    #[arg(short, long, value_enum)]
    pub algorithm: Option<OTPAlgorithm>,

    /// New code length (1-10 characters)
    #[arg(short, long, value_parser=value_parser!(u64).range(1..=10))]
    pub digits: Option<u64>,

    /// New code refresh period in seconds
    #[arg(short = 'e', long)]
    pub period: Option<u64>,

    /// New HOTP counter
    #[arg(short, long)]
    pub counter: Option<u64>,

    /// New PIN for a Yandex or MOTP account
    #[arg(short, long)]
    pub pin: Option<String>,

    /// Prompt for a new secret key
    #[arg(short = 'k', long = "change-secret")]
    pub change_secret: bool,
}

impl SubcommandExecutor for EditArgs {
    fn run_command(self, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let secret = self
            .change_secret
            .then(|| rpassword::prompt_password("New secret key: ").unwrap());

        // User provides row number from dashboard which is equal to the array index plus one
        let index = self.index;

        if let Some(real_index) = index.checked_sub(1) {
            if real_index >= database.elements_ref().len() {
                return Err(eyre!(
                    "Invalid account index {index}. Use cotp list to see account indexes"
                ));
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
                None => return Err(eyre!("No account found at index {index}")),
            }
            Ok(database)
        } else {
            Err(eyre!(
                "Invalid account index {index}. Use cotp list to see account indexes"
            ))
        }
    }
}
