//! Optional YubiKey OATH backend. ykman owns device transport and touch prompts.
use clap::{Args, Subcommand, ValueEnum};
use color_eyre::eyre::{Result, WrapErr, eyre};
use std::io::IsTerminal;
use std::process::{Command, Stdio};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Backend {
    #[default]
    Vault,
    Yubikey,
}

#[derive(Args)]
pub struct HardwareArgs {
    #[command(subcommand)]
    command: HardwareCommand,
}

#[derive(Subcommand)]
enum HardwareCommand {
    /// List accounts stored on the selected key
    List {
        #[arg(long)]
        json: bool,
    },
    /// Generate exactly one code (HOTP advances on the device)
    Code {
        query: String,
        #[arg(long)]
        copy: bool,
        #[arg(long)]
        quiet: bool,
        #[arg(long)]
        json: bool,
    },
    /// Enroll an account; ykman securely prompts for its secret
    Add {
        name: String,
        #[arg(long)]
        touch: bool,
        #[arg(long)]
        issuer: Option<String>,
    },
    /// Delete a single matching account, with ykman's confirmation
    Delete { query: String },
}

impl crate::arguments::SubcommandExecutor for HardwareArgs {
    fn run_command(
        self,
        _: crate::otp::otp_element::OTPDatabase,
    ) -> Result<crate::otp::otp_element::OTPDatabase> {
        Err(eyre!(
            "Hardware commands must run before opening a local vault"
        ))
    }
}

fn command(device: Option<&str>) -> Command {
    let mut command = Command::new("ykman");
    if let Some(serial) = device {
        command.args(["--device", serial]);
    }
    command.args(["oath", "accounts"]);
    command.stderr(Stdio::inherit());
    if !std::io::stdin().is_terminal() {
        command.stdin(Stdio::null());
    }
    command
}

fn accounts(device: Option<&str>) -> Result<Vec<String>> {
    let output = command(device)
        .arg("list")
        .output()
        .wrap_err("Cannot run ykman; install YubiKey Manager and enable the OATH application")?;
    if !output.status.success() {
        return Err(eyre!("YubiKey account listing failed ({})", output.status));
    }
    let text = String::from_utf8(output.stdout).wrap_err("Invalid UTF-8 from ykman")?;
    Ok(text
        .lines()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect())
}

fn select<'a>(names: &'a [String], query: &str) -> Result<&'a str> {
    if query.is_empty() {
        return Err(eyre!("An account query is required"));
    }
    let exact: Vec<_> = names
        .iter()
        .filter(|n| n.eq_ignore_ascii_case(query))
        .collect();
    let matches: Vec<_> = if exact.is_empty() {
        let query = query.to_lowercase();
        names
            .iter()
            .filter(|n| n.to_lowercase().contains(&query))
            .collect()
    } else {
        exact
    };
    match matches.as_slice() {
        [name] => Ok(name),
        [] => Err(eyre!("No YubiKey account matches the query")),
        _ => Err(eyre!(
            "Ambiguous YubiKey account query ({} matches); use the full account name",
            matches.len()
        )),
    }
}

pub fn list(device: Option<&str>, json: bool) -> Result<()> {
    let names = accounts(device)?;
    if json {
        let entries: Vec<_> = names
            .iter()
            .map(|name| serde_json::json!({"id":name,"backend":"yubikey"}))
            .collect();
        println!("{}", serde_json::to_string(&entries)?);
    } else {
        for name in names {
            println!("{}", name.escape_default());
        }
    }
    Ok(())
}

pub fn code(device: Option<&str>, query: &str, copy: bool, quiet: bool, json: bool) -> Result<()> {
    if quiet && !copy {
        return Err(eyre!("--quiet requires --copy"));
    }
    let names = accounts(device)?;
    let name = select(&names, query)?;
    // --single also asks ykman to reject ambiguity before advancing an HOTP counter.
    let output = command(device)
        .args(["code", "--single", "--", name])
        .output()
        .wrap_err("Cannot run ykman")?;
    if !output.status.success() {
        return Err(eyre!("YubiKey code generation failed ({})", output.status));
    }
    let text = Zeroizing::new(
        String::from_utf8(output.stdout).wrap_err("Invalid code output from ykman")?,
    );
    let code = text.trim();
    if !(6..=8).contains(&code.len()) || !code.bytes().all(|b| b.is_ascii_digit()) {
        return Err(eyre!("ykman returned an unexpected code format"));
    }
    if copy {
        crate::clipboard::copy_string_to_clipboard(code)?;
    }
    if !quiet {
        if json {
            println!(
                "{}",
                serde_json::json!({"id":name,"backend":"yubikey","code":code,"expires_at":null})
            );
        } else {
            println!("{code}");
        }
    }
    Ok(())
}

pub fn handle(args: &crate::arguments::CotpArgs) -> Result<bool> {
    use crate::arguments::CotpSubcommands;
    let device = args.device.as_deref();
    if let Some(CotpSubcommands::Hardware(hardware)) = &args.command {
        match &hardware.command {
            HardwareCommand::List { json } => list(device, *json)?,
            HardwareCommand::Code {
                query,
                copy,
                quiet,
                json,
            } => code(device, query, *copy, *quiet, *json)?,
            HardwareCommand::Add {
                name,
                touch,
                issuer,
            } => {
                if !std::io::stdin().is_terminal() {
                    return Err(eyre!(
                        "Hardware enrollment requires an interactive terminal"
                    ));
                }
                let mut command = command(device);
                command.arg("add");
                if *touch {
                    command.arg("--touch");
                }
                if let Some(issuer) = issuer {
                    command.args(["--issuer", issuer]);
                }
                let status = command.args(["--", name]).status()?;
                if !status.success() {
                    return Err(eyre!("YubiKey enrollment failed ({status})"));
                }
            }
            HardwareCommand::Delete { query } => {
                if !std::io::stdin().is_terminal() {
                    return Err(eyre!("Hardware deletion requires an interactive terminal"));
                }
                let names = accounts(device)?;
                let name = select(&names, query)?;
                let status = command(device).args(["delete", "--", name]).status()?;
                if !status.success() {
                    return Err(eyre!("YubiKey deletion failed ({status})"));
                }
            }
        }
        return Ok(true);
    }
    if args.backend == Backend::Vault {
        return Ok(false);
    }
    match &args.command {
        Some(CotpSubcommands::List(list_args)) => {
            list(device, list_args.format.as_ref().is_some_and(|f| f.json))?
        }
        _ => {
            return Err(eyre!(
                "Use cotp hardware code QUERY or cotp --backend yubikey list for hardware accounts"
            ));
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ambiguous_hardware_queries_fail_before_generation() {
        let names = vec!["GitHub:personal".into(), "GitHub:work".into()];
        assert!(select(&names, "github").is_err());
        assert_eq!(select(&names, "GITHUB:work").unwrap(), "GitHub:work");
        assert!(select(&names, "").is_err());
        assert!(select(&names, "absent").is_err());
    }
    #[test]
    fn device_and_query_arguments_are_not_shell_commands() {
        let command = command(Some("123456"));
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, ["--device", "123456", "oath", "accounts"]);
        assert_eq!(command.get_program(), "ykman");
    }
}
