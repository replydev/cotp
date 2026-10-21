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
