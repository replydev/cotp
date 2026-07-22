use clap::Args;
use eyre::eyre;
use serde::Serialize;

use crate::otp::otp_element::{OTPDatabase, OTPElement};

use super::SubcommandExecutor;

#[derive(Args)]
pub struct ListArgs {
    /// List output format
    #[command(flatten)]
    pub format: Option<ListFormat>,
}

/// Defines the output formats of the list subcommand
#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct ListFormat {
    /// List OTP codes in plain format
    #[arg(short, long)]
    pub plain: bool,

    /// List OTP codes in JSON format
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

impl<'a> From<&'a OTPElement> for JsonOtpList<'a> {
    fn from(value: &'a OTPElement) -> Self {
        // Degrade per-element like the table view does: an uncomputable code
        // (e.g. HOTP missing its counter) must not make the whole listing fail
        let otp_code = value
            .get_otp_code()
            .unwrap_or_else(|error| error.to_string());
        JsonOtpList {
            issuer: &value.issuer,
            label: &value.label,
            otp_code,
        }
    }
}

const NO_ISSUER_TEXT: &str = "<No issuer>";

impl SubcommandExecutor for ListArgs {
    fn run_command(self, otp_database: OTPDatabase) -> eyre::Result<OTPDatabase> {
        if self.format.unwrap_or_default().json {
            let json_elements = otp_database
                .elements_ref()
                .iter()
                .map(Into::into)
                .collect::<Vec<JsonOtpList>>();

            let stringified = serde_json::to_string_pretty(&json_elements)
                .map_err(|e| eyre!("Error during JSON serialization: {:?}", e))?;
            println!("{stringified}");
        } else {
            if otp_database.elements_ref().is_empty() {
                println!("No elements to list");
                return Ok(otp_database);
            }

            const ISSUER_HEADER: &str = "Issuer";
            const LABEL_HEADER: &str = "Label";

            // Clamp column widths to at least the header lengths so short
            // issuers/labels can never underflow the padding computation
            let issuer_width = calculate_width(&otp_database, |element| {
                let issuer_length = element.issuer.chars().count();
                if issuer_length > 0 {
                    issuer_length
                } else {
                    NO_ISSUER_TEXT.chars().count()
                }
            })
            .max(ISSUER_HEADER.chars().count());

            let label_width =
                calculate_width(&otp_database, |element| element.label.chars().count())
                    .max(LABEL_HEADER.chars().count());

            println!(
                "{0: <6} {1: <issuer_width$} {2: <label_width$} {3: <10}",
                "Index", ISSUER_HEADER, LABEL_HEADER, "OTP",
            );
            otp_database
                .elements_ref()
                .iter()
                .enumerate()
                .for_each(|(index, e)| {
                    let issuer = if e.issuer.is_empty() {
                        NO_ISSUER_TEXT
                    } else {
                        e.issuer.as_str()
                    };
                    println!(
                        "{0: <6} {1: <issuer_width$} {2: <label_width$} {3: <10}",
                        index + 1,
                        issuer,
                        e.label,
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
        .elements_ref()
        .iter()
        .map(get_number_of_chars)
        .max()
        .unwrap_or_default()
        + 3
}
