# Configuration and shell integration

Run `cotp completions bash`, `cotp completions zsh`, `cotp completions fish`, or
`cotp completions powershell` and install the output using your shell's completion
loader. `cotp man` writes a roff manual to stdout; `cotp man --directory ./man`
generates pages for all commands. Neither command needs a vault or password.

Settings are JSON at the platform configuration directory under `cotp/config.json`.
Use `--config FILE` or `COTP_CONFIG` to choose another file. Unknown settings are
errors to catch spelling mistakes. Paths are literal: expand `~` in your shell or
use absolute paths in JSON. Relative vault paths resolve from the working directory.

```json
{
  "profiles": {"personal": "/home/me/.local/share/cotp/personal.cotp", "work": "/home/me/.local/share/cotp/work.cotp"},
  "default_profile": "personal",
  "hide_codes": true,
  "idle_timeout_seconds": 300,
  "clipboard_seconds": 30,
  "keychain": false
}
```

Choose a profile with `cotp --profile work` or `COTP_PROFILE=work cotp`.
Vault path precedence is `--database-path`, `COTP_DB_PATH`, selected profile,
configuration `database_path`, then the existing portable/platform default.
Explicit unknown profile names fail instead of opening the wrong vault.
Zero disables inactivity locking or clipboard expiration. Keychain integration is
opt-in; the password prompt remains the default.

For a build without native desktop clipboard dependencies:

```sh
cargo build --release --locked --no-default-features
```

This build retains the TUI and SSH OSC52 clipboard support. Local desktop copying
requires the default `desktop-clipboard` feature.

## Optional OS credential store unlock

By default cotp asks for your vault password. Set `"keychain": true` in your
configuration to permit initial unlock using an explicitly saved password:

```sh
cotp keychain remember
cotp keychain status
cotp keychain forget
```

`remember` requires the current vault password through a terminal prompt,
`--password-stdin`, or `--password-fd`. It authenticates the encrypted vault before
saving anything. Passwords are never accepted through command arguments or
environment variables. Enrollment is explicit: enabling the setting, creating a
vault, or typing a password during normal unlock never saves a credential.
`remember` and `forget` work even while automatic keychain unlock is disabled.
`status` reports presence and whether automatic unlock is enabled; it never prints
the stored password. `forget` deletes only the saved OS credential.

An explicit `--password-stdin` or `--password-fd` always takes precedence over the
saved password. Missing credentials fall back to the normal terminal prompt;
`--no-input` rejects that prompt. Credential store errors are reported and stop
unlock, rather than falling back silently. A stale password also fails normal vault
authentication: after changing your password, run `keychain remember` again or
`keychain forget`. New vault creation still requires an explicit password or the
usual verified prompt.

Credentials use service `cotp.vault-password.v1` and a SHA-256 hash of the canonical
vault path as the account identifier. Symlinks to the same vault share a credential;
distinct vault paths have distinct credentials. Moving a vault requires enrolling
its new path. Forget the old path before moving it. The OS credential store protects
the saved password under your operating system account; your vault file remains
encrypted normally. Inactivity locking clears the running session but preserves
the saved credential, and session unlock still requires entering the vault password.

Supported native stores are Linux Secret Service over D-Bus (for example GNOME
Keyring or KWallet with Secret Service enabled), macOS Keychain, and Windows
Credential Manager. Linux needs a running, unlocked Secret Service session; headless
systems can keep using stdin/file descriptor passwords. Other platforms explicitly
reject keychain commands. The `keyring` 3.6 API is compiled with target-specific
`sync-secret-service` + `crypto-rust` + `vendored`, `apple-native`, or `windows-native`
features; Linux includes D-Bus client code and encrypts secrets during bus transfer.
See the [keyring native store documentation](https://docs.rs/keyring/3.6.3/keyring/#credential-store-features)
and [entry API](https://docs.rs/keyring/3.6.3/keyring/struct.Entry.html).
