//! Explicit, local merging of authenticated encrypted backups. No network transport.
use crate::{
    arguments::SubcommandExecutor,
    otp::{
        otp_element::{OTPDatabase, OTPElement},
        otp_type::OTPType,
    },
};
use clap::{Args, ValueEnum};
use color_eyre::eyre::{Result, eyre};
use std::{
    io::{self, BufRead, IsTerminal, Read},
    path::PathBuf,
};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ConflictPolicy {
    #[default]
    Abort,
    KeepLocal,
    TakeRemote,
}
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum HotpPolicy {
    #[default]
    Reject,
    Max,
}

#[derive(Args)]
pub struct MergeArgs {
    /// An encrypted cotp vault or backup from another device
    file: PathBuf,
    /// Resolve changed accounts explicitly; no changes are written on abort
    #[arg(long, value_enum, default_value = "abort")]
    conflict: ConflictPolicy,
    /// HOTP is rejected unless max is chosen; never decreases an existing counter
    #[arg(long, value_enum, default_value = "reject")]
    hotp: HotpPolicy,
    /// Read the source vault's password from stdin (separate from destination unlock)
    #[arg(long)]
    source_password_stdin: bool,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Default, serde::Serialize)]
struct Report {
    added: usize,
    updated: usize,
    unchanged: usize,
    conflicts_kept: usize,
}

fn same_identity(a: &OTPElement, b: &OTPElement) -> bool {
    a.type_ == b.type_
        && a.secret == b.secret
        && a.algorithm == b.algorithm
        && a.digits == b.digits
        && a.period == b.period
        && a.pin == b.pin
        && a.issuer == b.issuer
        && a.label == b.label
}

fn merge(
    local: &OTPDatabase,
    remote: OTPDatabase,
    conflict: ConflictPolicy,
    hotp: HotpPolicy,
) -> Result<(OTPDatabase, Report)> {
    let mut candidate = OTPDatabase {
        version: local.version,
        elements: local.elements.clone(),
        needs_modification: false,
    };
    let mut report = Report::default();
    for mut incoming in remote.elements {
        if incoming.type_ == OTPType::Hotp && matches!(hotp, HotpPolicy::Reject) {
            return Err(eyre!(
                "Source contains HOTP accounts; choose --hotp max after designating a single device to generate HOTP codes"
            ));
        }
        let existing = candidate
            .elements
            .iter()
            .position(|e| e.id == incoming.id)
            .or_else(|| {
                candidate
                    .elements
                    .iter()
                    .position(|e| same_identity(e, &incoming))
            });
        if let Some(index) = existing {
            let old = &candidate.elements[index];
            incoming.id = old.id.clone();
            let counter = if incoming.type_ == OTPType::Hotp && same_identity(old, &incoming) {
                old.counter.max(incoming.counter)
            } else {
                incoming.counter
            };
            // Counters are merged independently of metadata conflict policy.
            let mut comparable = incoming.clone();
            comparable.counter = old.counter;
            if &comparable == old {
                if old.counter != counter {
                    candidate.elements[index].counter = counter;
                    candidate.mark_modified();
                    report.updated += 1;
                } else {
                    report.unchanged += 1;
                }
                continue;
            }
            match conflict {
                ConflictPolicy::Abort => {
                    return Err(eyre!(
                        "Conflicting account {}; use --conflict keep-local or take-remote",
                        old.id
                    ));
                }
                ConflictPolicy::KeepLocal => {
                    if old.type_ == OTPType::Hotp
                        && same_identity(old, &incoming)
                        && old.counter != counter
                    {
                        candidate.elements[index].counter = counter;
                        candidate.mark_modified();
                        report.updated += 1;
                    }
                    report.conflicts_kept += 1;
                }
                ConflictPolicy::TakeRemote => {
                    incoming.counter = counter;
                    candidate.elements[index] = incoming;
                    candidate.mark_modified();
                    report.updated += 1;
                }
            }
        } else {
            candidate.add_element(incoming)?;
            report.added += 1;
        }
    }
    let mut aliases = std::collections::HashSet::new();
    for element in &candidate.elements {
        if let Some(alias) = &element.alias
            && !aliases.insert(alias.to_lowercase())
        {
            return Err(eyre!(
                "Merge would create duplicate aliases; resolve aliases before merging"
            ));
        }
    }
    Ok((candidate, report))
}

impl SubcommandExecutor for MergeArgs {
    fn run_command(self, database: OTPDatabase) -> Result<OTPDatabase> {
        let password = Zeroizing::new(if self.source_password_stdin {
            io::stdin()
                .lock()
                .lines()
                .next()
                .ok_or_else(|| eyre!("Missing source password on stdin"))??
        } else {
            if !io::stdin().is_terminal() {
                return Err(eyre!(
                    "Use --source-password-stdin for noninteractive merge"
                ));
            }
            rpassword::prompt_password("Source backup password: ")?
        });
        let mut bytes = Vec::new();
        std::fs::File::open(&self.file)?
            .take(64 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(eyre!("Source backup exceeds 64 MiB"));
        }
        let remote = crate::backup::validate(&bytes, &password)?;
        let (candidate, report) = merge(&database, remote, self.conflict, self.hotp)?;
        if self.json {
            println!("{}", serde_json::to_string(&report)?);
        } else {
            eprintln!(
                "Merge{}: {} added, {} updated, {} unchanged, {} conflicts kept locally",
                if self.dry_run { " preview" } else { "" },
                report.added,
                report.updated,
                report.unchanged,
                report.conflicts_kept
            );
        }
        if self.dry_run {
            Ok(database)
        } else {
            Ok(candidate)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::otp_element::OTPElementBuilder;
    fn vault(label: &str) -> OTPDatabase {
        let mut db = OTPDatabase::default();
        db.add_element(
            OTPElementBuilder::default()
                .issuer("Example")
                .label(label)
                .secret("JBSWY3DP")
                .build()
                .unwrap(),
        )
        .unwrap();
        db.needs_modification = false;
        db
    }
    fn clone_vault(db: &OTPDatabase) -> OTPDatabase {
        OTPDatabase {
            version: db.version,
            elements: db.elements.clone(),
            needs_modification: false,
        }
    }
    #[test]
    fn conflicts_abort_without_mutating_local() {
        let local = vault("alice");
        let mut remote = clone_vault(&local);
        remote.elements[0].label = "renamed".into();
        assert!(merge(&local, remote, ConflictPolicy::Abort, HotpPolicy::Reject).is_err());
        assert_eq!(local.elements[0].label, "alice");
        assert!(!local.is_modified());
    }
    #[test]
    fn merging_same_snapshot_twice_is_idempotent() {
        let local = vault("alice");
        let remote = vault("bob");
        let again = clone_vault(&remote);
        let (merged, report) =
            merge(&local, remote, ConflictPolicy::Abort, HotpPolicy::Reject).unwrap();
        assert_eq!(report.added, 1);
        let (merged_again, report) =
            merge(&merged, again, ConflictPolicy::Abort, HotpPolicy::Reject).unwrap();
        assert_eq!(report.added, 0);
        assert_eq!(merged_again.elements.len(), 2);
    }
    #[test]
    fn hotp_requires_policy_and_never_moves_backwards() {
        let mut local = vault("alice");
        local.elements[0].type_ = OTPType::Hotp;
        local.elements[0].counter = Some(50);
        let mut remote = clone_vault(&local);
        remote.elements[0].counter = Some(4);
        assert!(
            merge(
                &local,
                clone_vault(&remote),
                ConflictPolicy::TakeRemote,
                HotpPolicy::Reject
            )
            .is_err()
        );
        let (merged, _) =
            merge(&local, remote, ConflictPolicy::TakeRemote, HotpPolicy::Max).unwrap();
        assert_eq!(merged.elements[0].counter, Some(50));
        let mut remote = clone_vault(&local);
        remote.elements[0].counter = Some(70);
        let (merged, _) =
            merge(&local, remote, ConflictPolicy::KeepLocal, HotpPolicy::Max).unwrap();
        assert_eq!(merged.elements[0].counter, Some(70));
    }
    #[test]
    fn explicit_remote_policy_preserves_account_identity() {
        let local = vault("alice");
        let mut remote = clone_vault(&local);
        remote.elements[0].label = "new label".into();
        let (merged, report) = merge(
            &local,
            remote,
            ConflictPolicy::TakeRemote,
            HotpPolicy::Reject,
        )
        .unwrap();
        assert_eq!(merged.elements[0].id, local.elements[0].id);
        assert_eq!(merged.elements[0].label, "new label");
        assert_eq!(report.updated, 1);
    }
}
