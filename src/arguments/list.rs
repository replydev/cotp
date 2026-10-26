use clap::Args;
use color_eyre::eyre::{Result, eyre};
use serde::Serialize;

use crate::otp::otp_element::{OTPDatabase, OTPElement};

use super::SubcommandExecutor;

#[derive(Args)]
#[command(after_help = "EXAMPLES
  cotp list
  cotp list --json

Shows current codes once and exits. Run cotp without a command for the live dashboard.")]
pub struct ListArgs {
    /// List output format
    #[command(flatten)]
    pub format: Option<ListFormat>,
}

/// Defines the output formats of the list subcommand
#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct ListFormat {
    /// Show codes in a table (default)
    #[arg(short, long)]
    pub plain: bool,

    /// Output codes as JSON for scripts
    #[arg(short = 'e', long)]
    pub json: bool,
}

impl Default for ListFormat {
    fn default() -> Self {
        Self {
            plain: true,
            json: false,
        }
    }
}

/// Defines JSON structure to output using the --json argument in the list subcommand
#[derive(Serialize)]
struct JsonOtpList<'a> {
    issuer: &'a str,
    label: &'a str,
    otp_code: String,
}

impl<'a> TryFrom<&'a OTPElement> for JsonOtpList<'a> {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: &'a OTPElement) -> Result<Self, Self::Error> {
        let otp_code = value.get_otp_code()?;
        Ok(JsonOtpList {
            issuer: &value.issuer,
            label: &value.label,
            otp_code,
        })
    }
}

const NO_ISSUER_TEXT: &str = "<No issuer>";

impl SubcommandExecutor for ListArgs {
    fn run_command(self, otp_database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        if self.format.unwrap_or_default().json {
            let json_elements = otp_database
                .elements
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<JsonOtpList>>>()?;

            let stringified = serde_json::to_string_pretty(&json_elements)
                .map_err(|e| eyre!("Could not format codes as JSON: {}", e))?;
            print!("{stringified}");
        } else {
            if otp_database.elements.is_empty() {
                println!("The vault has no accounts. Use cotp add --help to add one");
                return Ok(otp_database);
            }
            let issuer_width = calculate_width(&otp_database, |element| {
                let issuer_length = element.issuer.chars().count();
                if issuer_length > 0 {
                    issuer_length
                } else {
                    NO_ISSUER_TEXT.chars().count()
                }
            });

            let label_width =
                calculate_width(&otp_database, |element| element.label.chars().count());

            println!(
                "{0: <6} {1} {2} {3: <10}",
                "Index",
                "Issuer".to_owned() + " ".repeat(issuer_width - 6).as_ref(),
                "Label".to_owned() + " ".repeat(label_width - 5).as_ref(),
                "OTP",
            );
            otp_database
                .elements
                .iter()
                .enumerate()
                .for_each(|(index, e)| {
                    println!(
                        "{0: <6} {1} {2} {3: <10}",
                        index + 1,
                        if e.issuer.is_empty() {
                            NO_ISSUER_TEXT.to_owned()
                                + " "
                                    .repeat(issuer_width - NO_ISSUER_TEXT.chars().count())
                                    .as_str()
                        } else {
                            e.issuer.clone()
                                + " ".repeat(issuer_width - e.issuer.chars().count()).as_str()
                        },
                        e.label.clone()
                            + " ".repeat(label_width - e.label.chars().count()).as_str(),
                        e.get_otp_code().unwrap_or("ERROR".to_string())
                    );
                });
        }

        Ok(otp_database)
    }
}

fn calculate_width<F>(otp_database: &OTPDatabase, get_number_of_chars: F) -> usize
where
    F: Fn(&OTPElement) -> usize,
{
    otp_database
        .elements
        .iter()
        .map(get_number_of_chars)
        .max()
        .unwrap_or_default()
        + 3
}
