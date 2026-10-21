use clap::{Args, value_parser};

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPDatabase};

use super::{
    SubcommandExecutor,
    selector::{AccountSelector, validate_alias},
};

#[derive(Args)]
pub struct EditArgs {
    /// One-based account index shown by list
    #[arg(short, long, required_unless_present_any = ["id", "alias", "match_issuer", "match_label"])]
    pub index: Option<usize>,
    /// Persistent account identifier
    #[arg(long)]
    pub id: Option<String>,
    /// Existing account alias
    #[arg(long)]
    pub alias: Option<String>,
    /// Select by issuer (case-insensitive glob)
    #[arg(long)]
    pub match_issuer: Option<String>,
    /// Select by label (case-insensitive glob)
    #[arg(long)]
    pub match_label: Option<String>,
    /// Set a unique account alias
    #[arg(long, conflicts_with = "clear_alias")]
    pub set_alias: Option<String>,
    /// Remove the account alias
    #[arg(long)]
    pub clear_alias: bool,
    /// New code issuer
    #[arg(short = 's', long)]
    pub issuer: Option<String>,
    /// New code label
    #[arg(short, long)]
    pub label: Option<String>,
    /// OTP algorithm
    #[arg(short, long, value_enum)]
    pub algorithm: Option<OTPAlgorithm>,
    /// Code digits
    #[arg(short, long, value_parser=value_parser!(u64).range(1..=10))]
    pub digits: Option<u64>,
    /// Code period
    #[arg(short = 'e', long, value_parser=value_parser!(u64).range(1..))]
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
        let index = AccountSelector {
            index: self.index,
            id: self.id,
            alias: self.alias,
            issuer: self.match_issuer,
            label: self.match_label,
            query: None,
        }
        .select(&database)?;
        if let Some(alias) = &self.set_alias {
            validate_alias(&database, alias, Some(index))?;
        }
        let secret = self
            .change_secret
            .then(|| crate::utils::prompt_password("Insert the secret: "))
            .transpose()?;
        let element = database
            .mut_element(index)
            .expect("selected account exists");
        if let Some(value) = self.issuer {
            element.issuer = value;
        }
        if let Some(value) = self.label {
            element.label = value;
        }
        if let Some(value) = self.digits {
            element.digits = value;
        }
        if let Some(value) = self.period {
            element.period = value;
        }
        if let Some(value) = self.algorithm {
            element.algorithm = value;
        }
        if self.counter.is_some() {
            element.counter = self.counter;
        }
        if self.pin.is_some() {
            element.pin = self.pin;
        }
        if let Some(value) = secret {
            element.secret = value;
        }
        if let Some(value) = self.set_alias {
            element.alias = Some(value);
        }
        if self.clear_alias {
            element.alias = None;
        }
        database.mark_modified();
        Ok(database)
    }
}
