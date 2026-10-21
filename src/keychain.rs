//! Explicitly enrolled vault passwords in native OS credential stores.
use crate::{arguments::CotpArgs, otp::otp_element::OTPDatabase, utils};
use clap::{Args, Subcommand};
use color_eyre::eyre::{Result, WrapErr, eyre};
use sha2::{Digest, Sha256};
use std::path::Path;
use zeroize::Zeroizing;

const SERVICE: &str = "cotp.vault-password.v1";

#[derive(Args)]
pub struct KeychainArgs {
    #[command(subcommand)]
    action: KeychainAction,
}

#[derive(Subcommand)]
enum KeychainAction {
    /// Authenticate the vault password and save it in the OS credential store
    Remember,
    /// Delete this vault's saved password without unlocking the vault
    Forget,
    /// Report whether this vault has a saved password (never display the password)
    Status,
}

impl crate::arguments::SubcommandExecutor for KeychainArgs {
    fn run_command(self, _: OTPDatabase) -> Result<OTPDatabase> {
        Err(eyre!("Keychain commands must run before opening the vault"))
    }
}

trait CredentialStore {
    fn get(&self) -> Result<Option<Zeroizing<String>>>;
    fn set(&self, password: &str) -> Result<()>;
    fn delete(&self) -> Result<()>;
}

struct NativeStore(keyring::Entry);
impl NativeStore {
    fn open(path: &Path) -> Result<Self> {
        if !cfg!(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "windows"
        )) {
            return Err(eyre!(
                "OS keychain integration is unsupported on this platform"
            ));
        }
        keyring::Entry::new(SERVICE, &credential_id(path)?)
            .map(Self)
            .map_err(|_| backend_error())
    }
}
fn backend_error() -> color_eyre::eyre::Report {
    // Native errors can include credential attributes; keep all diagnostics redacted.
    eyre!(
        "Cannot access the OS credential store; check that it is running, unlocked, and permits cotp access"
    )
}
impl CredentialStore for NativeStore {
    fn get(&self) -> Result<Option<Zeroizing<String>>> {
        match self.0.get_password() {
            Ok(password) => Ok(Some(Zeroizing::new(password))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(backend_error()),
        }
    }
    fn set(&self, password: &str) -> Result<()> {
        self.0.set_password(password).map_err(|_| backend_error())
    }
    fn delete(&self) -> Result<()> {
        match self.0.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(backend_error()),
        }
    }
}

fn credential_id(path: &Path) -> Result<String> {
    let canonical = path
        .canonicalize()
        .wrap_err("Cannot resolve existing vault path for the OS credential store")?;
    let mut hash = Sha256::new();
    hash.update(b"cotp vault path v1\0");
    hash.update(canonical.as_os_str().as_encoded_bytes());
    Ok(hex::encode(hash.finalize()))
}

/// Explicit password input always takes precedence; creation never consults a keychain.
pub fn unlock_password(
    path: &Path,
    explicit: Option<String>,
    first_run: bool,
) -> Result<Zeroizing<String>> {
    resolve_password(
        explicit.map(Zeroizing::new),
        crate::config::settings().keychain,
        first_run,
        || NativeStore::open(path)?.get(),
        || {
            if first_run {
                utils::verified_password("Choose a password: ", 8).map(Zeroizing::new)
            } else {
                utils::prompt_password("Password: ").map(Zeroizing::new)
            }
        },
    )
}

fn resolve_password(
    explicit: Option<Zeroizing<String>>,
    enabled: bool,
    first_run: bool,
    read_saved: impl FnOnce() -> Result<Option<Zeroizing<String>>>,
    prompt: impl FnOnce() -> Result<Zeroizing<String>>,
) -> Result<Zeroizing<String>> {
    if let Some(password) = explicit {
        return Ok(password);
    }
    if enabled
        && !first_run
        && let Some(password) = read_saved()?
    {
        return Ok(password);
    }
    prompt()
}

fn remember(store: &impl CredentialStore, path: &Path, password: &str) -> Result<()> {
    // Authentication completes before any credential store write. No database is saved.
    let bytes = std::fs::read(path).wrap_err("Cannot read vault for password verification")?;
    crate::backup::validate(&bytes, password)
        .wrap_err("Vault password verification failed; nothing was stored")?;
    store.set(password)
}

pub fn handle(args: &CotpArgs) -> Result<bool> {
    let Some(crate::arguments::CotpSubcommands::Keychain(command)) = &args.command else {
        return Ok(false);
    };
    let path = crate::path::init_path(args);
    match command.action {
        KeychainAction::Remember => {
            // Never enroll an existing keychain credential implicitly: require authentication input.
            let password = Zeroizing::new(if args.password_from_stdin {
                utils::stdin_line()?
            } else if let Some(fd) = args.password_fd {
                utils::password_from_fd(fd)?
            } else {
                utils::prompt_password("Vault password to remember: ")?
            });
            let store = NativeStore::open(&path)?;
            remember(&store, &path, &password)?;
            println!("Vault password saved in the OS credential store");
        }
        KeychainAction::Forget => {
            NativeStore::open(&path)?.delete()?;
            println!("Saved vault password removed");
        }
        KeychainAction::Status => {
            let saved = NativeStore::open(&path)?.get()?.is_some();
            println!(
                "{}; automatic keychain unlock {}",
                if saved {
                    "Saved password present"
                } else {
                    "No saved password"
                },
                if crate::config::settings().keychain {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    struct MockStore {
        password: RefCell<Option<Zeroizing<String>>>,
        fail: Cell<bool>,
        reads: Cell<usize>,
        writes: Cell<usize>,
    }
    impl CredentialStore for MockStore {
        fn get(&self) -> Result<Option<Zeroizing<String>>> {
            self.reads.set(self.reads.get() + 1);
            if self.fail.get() {
                return Err(backend_error());
            }
            Ok(self.password.borrow().clone())
        }
        fn set(&self, password: &str) -> Result<()> {
            if self.fail.get() {
                return Err(backend_error());
            }
            self.writes.set(self.writes.get() + 1);
            *self.password.borrow_mut() = Some(Zeroizing::new(password.to_owned()));
            Ok(())
        }
        fn delete(&self) -> Result<()> {
            if self.fail.get() {
                return Err(backend_error());
            }
            self.password.borrow_mut().take();
            Ok(())
        }
    }

    #[test]
    fn explicit_password_default_mode_and_new_vaults_never_read_keychain() {
        let store = MockStore::default();
        store.fail.set(true);
        for (explicit, enabled, first_run, expected) in [
            (Some("explicit"), true, false, "explicit"),
            (None, false, false, "prompt"),
            (None, true, true, "prompt"),
        ] {
            let password = resolve_password(
                explicit.map(|s| Zeroizing::new(s.into())),
                enabled,
                first_run,
                || store.get(),
                || Ok(Zeroizing::new("prompt".into())),
            )
            .unwrap();
            assert_eq!(password.as_str(), expected);
        }
        assert_eq!(store.reads.get(), 0);
    }

    #[test]
    fn saved_password_unlocks_missing_falls_back_and_backend_error_propagates() {
        let store = MockStore::default();
        store.set("saved").unwrap();
        let saved = resolve_password(
            None,
            true,
            false,
            || store.get(),
            || panic!("must not prompt"),
        )
        .unwrap();
        assert_eq!(saved.as_str(), "saved");
        store.delete().unwrap();
        let prompted = resolve_password(
            None,
            true,
            false,
            || store.get(),
            || Ok(Zeroizing::new("prompt".into())),
        )
        .unwrap();
        assert_eq!(prompted.as_str(), "prompt");
        // Represents the same prompt rejection returned by utils::prompt_password in no-input mode.
        assert!(
            resolve_password(
                None,
                true,
                false,
                || store.get(),
                || Err(eyre!("Interactive input is disabled"))
            )
            .is_err()
        );
        store.fail.set(true);
        let failed = resolve_password(
            None,
            true,
            false,
            || store.get(),
            || panic!("backend failure must not prompt"),
        );
        assert!(failed.is_err());
    }

    #[test]
    fn remember_authenticates_before_write_and_forget_removes_only_credential() {
        let store = MockStore::default();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.cotp");
        std::fs::copy("test_samples/cli_integration_test/empty_database", &path).unwrap();
        let before = std::fs::read(&path).unwrap();
        assert!(remember(&store, &path, "wrong-password").is_err());
        assert_eq!(store.writes.get(), 0);
        remember(&store, &path, "12345678").unwrap();
        assert_eq!(store.writes.get(), 1);
        assert_eq!(store.get().unwrap().unwrap().as_str(), "12345678");
        store.delete().unwrap();
        assert!(store.get().unwrap().is_none());
        assert_eq!(std::fs::read(path).unwrap(), before);
    }

    #[test]
    fn credential_scope_is_canonical_and_different_for_distinct_vaults() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.cotp");
        let b = dir.path().join("b.cotp");
        std::fs::write(&a, b"a").unwrap();
        std::fs::write(&b, b"b").unwrap();
        let id = credential_id(&a).unwrap();
        assert_eq!(id.len(), 64);
        assert_eq!(id, credential_id(&dir.path().join("./a.cotp")).unwrap());
        assert_ne!(id, credential_id(&b).unwrap());
        #[cfg(unix)]
        {
            let alias = dir.path().join("alias.cotp");
            std::os::unix::fs::symlink(&a, &alias).unwrap();
            assert_eq!(id, credential_id(&alias).unwrap());
        }
    }
}
