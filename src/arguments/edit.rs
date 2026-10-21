use clap::{Args, value_parser};

use crate::otp::{otp_algorithm::OTPAlgorithm, otp_element::OTPDatabase};

use super::{
    SubcommandExecutor,
    selector::{AccountSelector, validate_alias},
};

#[derive(Args)]
pub struct EditArgs {
    /// One-based account index shown by list
    #[arg(short, long, required_unless_present_any = ["id", "alias", "match_issuer", "match_label", "tags", "favorite"])]
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
    /// Require this tag when selecting (repeat to require all tags)
    #[arg(long = "tag", value_parser = crate::otp::organization::parse_tag)]
    pub tags: Vec<String>,
    /// Select favorite accounts only
    #[arg(long)]
    pub favorite: bool,
    /// Set a unique account alias
    #[arg(long, conflicts_with = "clear_alias")]
    pub set_alias: Option<String>,
    /// Remove the account alias
    #[arg(long)]
    pub clear_alias: bool,
    /// Add an account tag (repeat for multiple tags)
    #[arg(long = "add-tag", value_parser = crate::otp::organization::parse_tag)]
    pub add_tags: Vec<String>,
    /// Remove an account tag (case-insensitive; repeat as needed)
    #[arg(long = "remove-tag", value_parser = crate::otp::organization::parse_tag)]
    pub remove_tags: Vec<String>,
    /// Remove all account tags before adding new tags
    #[arg(long)]
    pub clear_tags: bool,
    /// Set favorite status explicitly: true or false
    #[arg(long, action = clap::ArgAction::Set)]
    pub set_favorite: Option<bool>,
    /// Set a custom position (lowest first)
    #[arg(long, allow_hyphen_values = true, conflicts_with = "clear_order")]
    pub order: Option<i64>,
    /// Restore alphabetical position
    #[arg(long)]
    pub clear_order: bool,
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
            tags: self.tags,
            favorite: self.favorite,
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
        if self.clear_tags {
            element.tags.clear();
        }
        element.tags.retain(|tag| {
            !self
                .remove_tags
                .iter()
                .any(|value| value.eq_ignore_ascii_case(tag))
        });
        crate::otp::organization::add_tags(&mut element.tags, self.add_tags);
        if let Some(favorite) = self.set_favorite {
            element.favorite = favorite;
        }
        if self.clear_order {
            element.order = None;
        } else if self.order.is_some() {
            element.order = self.order;
        }
        database.mark_modified();
        Ok(database)
    }
}
