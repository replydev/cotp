use clap::Args;
use color_eyre::eyre::Result;
use serde::Serialize;
use std::io::{self, Write};

use super::SubcommandExecutor;
use crate::otp::otp_element::OTPDatabase;

#[derive(Args)]
pub struct ListArgs {
    /// Require this tag (case-insensitive; repeat to require all tags)
    #[arg(long = "tag", value_parser = crate::otp::organization::parse_tag)]
    pub tags: Vec<String>,
    /// List favorite accounts only
    #[arg(long)]
    pub favorite: bool,
    /// List output format
    #[command(flatten)]
    pub format: Option<ListFormat>,
    /// List account metadata without generating or exposing codes
    #[arg(long)]
    pub metadata_only: bool,
}

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

#[derive(Serialize)]
struct JsonOtpList<'a> {
    index: usize,
    id: &'a str,
    alias: Option<&'a str>,
    issuer: &'a str,
    label: &'a str,
    tags: &'a [String],
    favorite: bool,
    order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    otp_code: Option<String>,
}

impl SubcommandExecutor for ListArgs {
    fn run_command(self, database: OTPDatabase) -> Result<OTPDatabase> {
        let elements = database
            .elements_ref()
            .iter()
            .enumerate()
            .filter(|(_, element)| {
                crate::otp::organization::matches_filters(element, &self.tags, self.favorite)
            })
            .map(|(index, element)| {
                Ok(JsonOtpList {
                    index: index + 1,
                    id: &element.id,
                    alias: element.alias.as_deref(),
                    issuer: &element.issuer,
                    label: &element.label,
                    tags: &element.tags,
                    favorite: element.favorite,
                    order: element.order,
                    otp_code: if self.metadata_only {
                        None
                    } else {
                        Some(element.get_otp_code()?)
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let mut stdout = io::stdout().lock();
        if self.format.unwrap_or_default().json {
            serde_json::to_writer_pretty(&mut stdout, &elements)?;
            writeln!(stdout)?;
        } else {
            write!(
                stdout,
                "Index\tID\tAlias\tIssuer\tLabel\tTags\tFavorite\tOrder"
            )?;
            if !self.metadata_only {
                write!(stdout, "\tOTP")?;
            }
            writeln!(stdout)?;
            for element in elements {
                write!(
                    stdout,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    element.index,
                    element.id.escape_default(),
                    element.alias.unwrap_or("").escape_default(),
                    element.issuer.escape_default(),
                    element.label.escape_default(),
                    element.tags.join(",").escape_default(),
                    element.favorite,
                    element
                        .order
                        .map(|value| value.to_string())
                        .unwrap_or_default()
                )?;
                if let Some(code) = element.otp_code {
                    write!(stdout, "\t{code}")?;
                }
                writeln!(stdout)?;
            }
        }
        Ok(database)
    }
}
