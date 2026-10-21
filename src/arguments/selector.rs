//! Shared single-account selection. Indices always match the one-based list display.
use clap::Args;
use color_eyre::eyre::eyre;
use globset::{GlobBuilder, GlobMatcher};

use crate::otp::otp_element::{OTPDatabase, OTPElement};

#[derive(Args, Clone, Default, Debug)]
#[group(id = "account_selector", required = true, multiple = true)]
pub struct AccountSelector {
    /// One-based account index shown by list
    #[arg(short, long)]
    pub index: Option<usize>,
    /// Persistent account identifier
    #[arg(long)]
    pub id: Option<String>,
    /// Exact account alias (case-insensitive)
    #[arg(long)]
    pub alias: Option<String>,
    /// Account issuer, optionally a case-insensitive glob
    #[arg(short = 's', long)]
    pub issuer: Option<String>,
    /// Account label, optionally a case-insensitive glob
    #[arg(short, long)]
    pub label: Option<String>,
    /// Account ID, alias, or text in its issuer/label
    #[arg(value_name = "QUERY")]
    pub query: Option<String>,
}

impl AccountSelector {
    pub fn select(&self, database: &OTPDatabase) -> color_eyre::Result<usize> {
        if self.index == Some(0) {
            return Err(eyre!("Account indices start at 1"));
        }
        if self.index.is_none()
            && self.id.is_none()
            && self.alias.is_none()
            && self.issuer.is_none()
            && self.label.is_none()
            && self.query.is_none()
        {
            return Err(eyre!(
                "Specify an account ID, alias, index, issuer, label, or query"
            ));
        }
        if [
            &self.id,
            &self.alias,
            &self.issuer,
            &self.label,
            &self.query,
        ]
        .iter()
        .any(|value| value.as_ref().is_some_and(|value| value.trim().is_empty()))
        {
            return Err(eyre!("Account selectors cannot be empty"));
        }
        let issuer = self.issuer.as_deref().map(matcher).transpose()?;
        let label = self.label.as_deref().map(matcher).transpose()?;
        let query = self.query.as_ref().map(|value| value.to_lowercase());
        let matches: Vec<usize> = database
            .elements_ref()
            .iter()
            .enumerate()
            .filter(|(index, element)| {
                self.index.is_none_or(|value| value == index + 1)
                    && self.id.as_ref().is_none_or(|value| *value == element.id)
                    && self.alias.as_ref().is_none_or(|value| {
                        element
                            .alias
                            .as_ref()
                            .is_some_and(|alias| alias.eq_ignore_ascii_case(value))
                    })
                    && issuer
                        .as_ref()
                        .is_none_or(|filter| filter.is_match(&element.issuer))
                    && label
                        .as_ref()
                        .is_none_or(|filter| filter.is_match(&element.label))
                    && query
                        .as_ref()
                        .is_none_or(|query| matches_query(element, query))
            })
            .map(|(index, _)| index)
            .collect();
        match matches.as_slice() {
            [index] => Ok(*index),
            [] => Err(eyre!("No account matches these selectors")),
            _ => Err(eyre!(
                "Ambiguous account selection: {} accounts match; use --id, --alias, or additional selectors",
                matches.len()
            )),
        }
    }
}

fn matcher(pattern: &str) -> color_eyre::Result<GlobMatcher> {
    Ok(GlobBuilder::new(pattern)
        .case_insensitive(true)
        .build()?
        .compile_matcher())
}

fn matches_query(element: &OTPElement, query: &str) -> bool {
    element.id.eq_ignore_ascii_case(query)
        || element
            .alias
            .as_ref()
            .is_some_and(|alias| alias.eq_ignore_ascii_case(query))
        || element.issuer.to_lowercase().contains(query)
        || element.label.to_lowercase().contains(query)
}

pub fn validate_alias(
    database: &OTPDatabase,
    alias: &str,
    except: Option<usize>,
) -> color_eyre::Result<()> {
    if alias.trim().is_empty() {
        return Err(eyre!("An alias cannot be empty"));
    }
    if database
        .elements_ref()
        .iter()
        .enumerate()
        .any(|(index, element)| {
            Some(index) != except
                && element
                    .alias
                    .as_ref()
                    .is_some_and(|existing| existing.eq_ignore_ascii_case(alias))
        })
    {
        return Err(eyre!("Alias is already assigned to another account"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::otp_element::OTPElementBuilder;

    fn database() -> OTPDatabase {
        OTPDatabase::from(vec![
            OTPElementBuilder::default()
                .secret("AA")
                .issuer("GitHub")
                .label("alice")
                .id("first")
                .alias(Some("work".into()))
                .build()
                .unwrap(),
            OTPElementBuilder::default()
                .secret("AA")
                .issuer("GitHub")
                .label("bob")
                .id("second")
                .build()
                .unwrap(),
        ])
    }

    #[test]
    fn indices_match_the_display_and_zero_is_rejected() {
        let database = database();
        for (input, output) in [(1, 0), (2, 1)] {
            assert_eq!(
                AccountSelector {
                    index: Some(input),
                    ..Default::default()
                }
                .select(&database)
                .unwrap(),
                output
            );
        }
        for input in [0, 3] {
            assert!(
                AccountSelector {
                    index: Some(input),
                    ..Default::default()
                }
                .select(&database)
                .is_err()
            );
        }
    }

    #[test]
    fn ambiguous_globs_are_rejected_and_filters_intersect() {
        let database = database();
        let mut selector = AccountSelector {
            issuer: Some("gIt*".into()),
            ..Default::default()
        };
        assert!(
            selector
                .select(&database)
                .unwrap_err()
                .to_string()
                .contains("Ambiguous")
        );
        selector.label = Some("a*".into());
        assert_eq!(selector.select(&database).unwrap(), 0);
        selector.index = Some(2);
        assert!(selector.select(&database).is_err());
    }

    #[test]
    fn ids_aliases_and_queries_select_accounts() {
        let database = database();
        for selector in [
            AccountSelector {
                id: Some("first".into()),
                ..Default::default()
            },
            AccountSelector {
                alias: Some("WORK".into()),
                ..Default::default()
            },
            AccountSelector {
                query: Some("ali".into()),
                ..Default::default()
            },
        ] {
            assert_eq!(selector.select(&database).unwrap(), 0);
        }
        assert!(AccountSelector::default().select(&database).is_err());
        assert!(
            AccountSelector {
                query: Some(String::new()),
                ..Default::default()
            }
            .select(&database)
            .is_err()
        );
        assert!(
            AccountSelector {
                issuer: Some("[".into()),
                ..Default::default()
            }
            .select(&database)
            .is_err()
        );
    }

    #[test]
    fn aliases_are_unique_ignoring_case_but_can_be_kept_by_their_owner() {
        let database = database();
        assert!(validate_alias(&database, "Work", None).is_err());
        assert!(validate_alias(&database, "Work", Some(0)).is_ok());
        assert!(validate_alias(&database, "", None).is_err());
    }
}
