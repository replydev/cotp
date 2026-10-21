use super::{SubcommandExecutor, selector::AccountSelector};
use crate::{
    clipboard,
    otp::{
        otp_element::{OTPDatabase, OTPElement},
        otp_type::OTPType,
    },
};
use clap::Args;
use color_eyre::eyre::eyre;
use serde::Serialize;
use std::{
    io::{self, Write},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Args)]
pub struct CodeArgs {
    #[command(flatten)]
    pub selector: AccountSelector,
    /// Emit one JSON object per code, including expiry in Unix seconds
    #[arg(long)]
    pub json: bool,
    /// Wait until the code has this much validity left (e.g. 10s)
    #[arg(long, default_value="0s", value_parser=parse_duration)]
    pub min_validity: Duration,
    /// Emit a new code at each time interval until interrupted
    #[arg(long)]
    pub watch: bool,
    /// Copy the generated code to the clipboard
    #[arg(long, alias = "copy-clipboard")]
    pub copy: bool,
    /// Suppress code output when copying
    #[arg(long, requires = "copy", conflicts_with = "json")]
    pub quiet: bool,
    /// Show the current HOTP code without advancing its counter
    #[arg(long)]
    pub preview: bool,
}

#[derive(Serialize)]
struct CodeOutput<'a> {
    id: &'a str,
    issuer: &'a str,
    label: &'a str,
    code: &'a str,
    expires_at: Option<u64>,
    remaining_seconds: Option<u64>,
}

pub fn parse_duration(value: &str) -> Result<Duration, String> {
    let value = value.strip_suffix('s').unwrap_or(value);
    value
        .parse::<u64>()
        .map(Duration::from_secs)
        .map_err(|_| "Use a non-negative number of seconds, for example 10s".into())
}

pub(crate) fn period(element: &OTPElement) -> color_eyre::Result<Option<u64>> {
    match element.type_ {
        OTPType::Hotp => Ok(None),
        OTPType::Steam => Ok(Some(30)),
        _ if element.period == 0 => Err(eyre!("Account has an invalid zero period")),
        _ => Ok(Some(element.period)),
    }
}

fn now() -> color_eyre::Result<Duration> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?)
}

fn timing(time: Duration, period: u64) -> color_eyre::Result<(u64, Duration)> {
    let expires = (time.as_secs() / period)
        .checked_add(1)
        .and_then(|next| next.checked_mul(period))
        .ok_or_else(|| eyre!("Code expiry overflows the clock"))?;
    Ok((expires, Duration::from_secs(expires).saturating_sub(time)))
}

impl SubcommandExecutor for CodeArgs {
    fn run_command(self, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let index = self.selector.select(&database)?;
        let period = period(&database.elements_ref()[index])?;
        if period.is_none() && (self.watch || !self.min_validity.is_zero()) {
            return Err(eyre!(
                "HOTP has no time interval; --watch and --min-validity are unavailable"
            ));
        }
        if period.is_some_and(|period| self.min_validity >= Duration::from_secs(period)) {
            return Err(eyre!(
                "Minimum validity must be shorter than the account period"
            ));
        }
        let mut previous_expiry = None;
        loop {
            let before = now()?;
            let timing_before = period.map(|period| timing(before, period)).transpose()?;
            if let Some((expiry, remaining)) = timing_before
                && (remaining < self.min_validity || previous_expiry == Some(expiry))
            {
                thread::sleep(remaining.min(Duration::from_millis(200)));
                continue;
            }
            let code = if period.is_none() && !self.preview {
                crate::persistence::consume_hotp(&mut database, index)?
            } else {
                database.elements_ref()[index].get_otp_code()?
            };
            let timing_after = period.map(|period| timing(now()?, period)).transpose()?;
            // A code generated across a rollover must never be paired with the next expiry.
            if timing_before.map(|value| value.0) != timing_after.map(|value| value.0)
                || timing_after.is_some_and(|(_, remaining)| remaining < self.min_validity)
            {
                continue;
            }
            if self.copy {
                clipboard::copy_string_to_clipboard(&code)?;
            }
            if !self.quiet {
                let mut stdout = io::stdout().lock();
                if self.json {
                    let element = &database.elements_ref()[index];
                    serde_json::to_writer(
                        &mut stdout,
                        &CodeOutput {
                            id: &element.id,
                            issuer: &element.issuer,
                            label: &element.label,
                            code: &code,
                            expires_at: timing_after.map(|value| value.0),
                            remaining_seconds: timing_after.map(|value| value.1.as_secs()),
                        },
                    )?;
                    writeln!(stdout)?;
                } else {
                    writeln!(stdout, "{code}")?;
                }
                stdout.flush()?;
            }
            if !self.watch {
                break;
            }
            previous_expiry = timing_after.map(|value| value.0);
        }
        Ok(database)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timing_respects_boundaries_without_overstating_validity() {
        assert_eq!(
            timing(Duration::from_secs(60), 30).unwrap(),
            (90, Duration::from_secs(30))
        );
        assert_eq!(
            timing(Duration::from_millis(89999), 30).unwrap(),
            (90, Duration::from_millis(1))
        );
        assert_eq!(
            timing(Duration::from_secs(90), 30).unwrap(),
            (120, Duration::from_secs(30))
        );
    }
    #[test]
    fn durations_have_an_explicit_safe_grammar() {
        assert_eq!(parse_duration("10s").unwrap(), Duration::from_secs(10));
        assert_eq!(parse_duration("0").unwrap(), Duration::ZERO);
        for invalid in ["-1s", "NaN", "10m", "18446744073709551616s"] {
            assert!(parse_duration(invalid).is_err());
        }
    }
}
