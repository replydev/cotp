# cotp - command line TOTP/HOTP authenticator

[![Actions Status](https://github.com/replydev/cotp/workflows/Build/badge.svg)](https://github.com/replydev/cotp/actions)
[![crates.io](https://img.shields.io/crates/v/cotp)](https://crates.io/crates/cotp) 
[![Downloads](https://img.shields.io/crates/d/cotp)](https://crates.io/crates/cotp)

[![Packaging status](https://repology.org/badge/vertical-allrepos/cotp.svg)](https://repology.org/project/cotp/versions)

I believe that security is of paramount importance, especially in this digital world. I created cotp because I needed a
minimalist, secure, desktop accessible software to manage my two-factor authentication codes.

# Overview

## Interface

<p align="center"><img src="/img/cotp_demo.gif" alt="cotp demo: navigate and search accounts, edit and add an account, hide codes, and confirm saving changes"/></p>

Run `cotp` to open your vault. The dashboard adapts to your terminal, with searchable
accounts, a countdown for the selected code, grouped codes, and an account detail panel
on wide screens. On your first run, choose a password to initialize the encrypted database.

Use the keyboard for every action, or click accounts, the search bar, and the visible
shortcuts. Keyboard and mouse-wheel navigation stop at the first and last account.
The `?` menu also offers clickable actions. No special icon font is required. The dashboard needs an interactive terminal;
use `cotp list` or `cotp list --json` when piping output.

| Key | Action |
| --- | --- |
| `↑` / `↓`, `k` / `j` | Select an account |
| `Home` / `End`, `PgUp` / `PgDn` | First / last account, or move ten accounts |
| `/` or `Ctrl+F` | Filter by issuer or account label |
| `Enter` | Copy the selected code; on the QR screen, copy its OTP URI |
| `a` / `e` | Add / edit an account |
| `Space` | Show or close the selected account's QR code |
| `h` | Hide / reveal codes |
| `f` | Toggle the selected favorite |
| `+` / `-` | Increase / decrease an HOTP counter |
| `d` or `Delete` | Delete an account, with confirmation |
| `?` or `i` | Open the shortcuts and actions menu |
| `Esc` | Close the current screen, finish searching, clear a filter, or quit |
| `Ctrl+L` | Lock immediately; enter the vault password to restore pending edits |
| `Ctrl+S` | Save pending account changes without quitting |
| `q`, `Ctrl+C`, `Ctrl+D` | Quit; account changes prompt you to save or discard |

### Stable account selection

Every account has a persistent random ID, available in `cotp list --json`.
Existing vaults upgrade automatically while preserving their secrets. All CLI
indices start at 1 and match the list display. Issuer and label selectors use
case-insensitive glob matching; multiple filters must all match. Commands fail
when the selection is ambiguous instead of choosing the first account.

```sh
cotp add --issuer GitHub --label you --alias work-github
cotp extract --alias work-github
cotp extract --issuer 'Git*' --label you
cotp edit --alias work-github --set-alias github --label new-name
cotp edit --match-issuer GitHub --match-label you --digits 8
cotp delete --id <account-id>
```

On `edit`, `--issuer` and `--label` still set new values; use `--match-issuer`
and `--match-label` to select by those fields. Aliases are unique ignoring case;
`edit --clear-alias` removes an alias. Account IDs survive renaming and TUI edits.

### Tags, favorites, and ordering

Add metadata with `add --tag work --tag mail --favorite --order 1`.
Tags match without case sensitivity; repeat `--tag` on `list`, `code`, `extract`,
`delete`, or `edit` to require all tags. `--favorite` restricts the selection to favorites.
Filtered list indices retain their positions in the full list.

```sh
cotp edit --alias github --add-tag work --remove-tag personal --set-favorite true
cotp edit --alias github --order -1
cotp list --tag work --favorite --metadata-only --json
cotp code --tag work --favorite
cotp edit --alias github --clear-tags --clear-order
```

`edit --set-favorite false` removes a favorite. Explicit order values sort first,
lowest to highest; other accounts sort by issuer, label, then persistent ID.
Favorites keep their existing positions. The dashboard displays favorites, aliases,
and tags; press `f` to toggle the selected favorite, then save or discard when quitting.
Dashboard and CLI queries support `tag:work`, `is:favorite`, and combinations such
as `tag:work is:favorite github`. Ordinary query text also matches aliases and IDs.

### Codes and scripts

`cotp code QUERY` selects one account by ID, alias, or issuer/label text.
Use `--json` for an object with `id`, `issuer`, `label`, `code`, `expires_at`
(Unix seconds), and `remaining` (whole seconds). HOTP expiry fields are `null`.
`--min-validity 10s` waits for a fresh interval when needed; the requested validity
must be shorter than the account period. `--watch` emits one code per interval,
flushing each line for consumers. `--copy --quiet` copies without printing a code.

HOTP `code` and legacy `extract` persist the next counter before releasing a code.
`code --preview` and `list` show the current counter without advancing it.
`list --metadata-only --json` lists account identities without generating codes.

For unattended use, pass `--no-input` to reject every interactive prompt.
`--password-stdin` reads one password line; on Unix, `--password-fd 3` keeps stdin
available for a secret or import. Passwords are never command-line values.
`init` creates a vault with a supplied password of at least eight characters and
refuses to overwrite an existing vault. Imports require `--yes` to apply without
confirmation; deletion uses `--yes`. Password rotation accepts `passwd --new-password-fd`.

```sh
cotp --no-input --password-fd 3 init 3<password-file
cotp --no-input --password-fd 3 import - --yes 3<password-file <accounts.txt
cotp --no-input --password-fd 3 code work --json --min-validity 10s 3<password-file
```

Command data goes to stdout; prompts, errors, and status go to stderr. Exit codes
are `0` for success, `1` for vault startup/configuration/final save errors, `2` for
invalid CLI arguments, and `3` for command failures (including selection, validation,
and HOTP transaction failures). Opening a legacy vault may persist its schema upgrade.

### Add and edit accounts

Press `a` or click **Add** to enter an issuer, account label, and secret. You can also
paste an `otpauth://` URI into the secret field to populate the form. Choose the OTP
type and algorithm, then adjust digits, period, counter, or PIN as required. Validation
focuses the field that needs attention.

<p align="center"><img src="/img/cotp-editor.svg" alt="cotp account editor with masked secret entry"/></p>

Use `Tab` / `Shift+Tab` or `↑` / `↓` to move through fields and buttons. Text fields
support cursor movement, `Home`, `End`, and `Ctrl+U` to clear. Click a type or algorithm
field, or use `←` / `→` or `Space`, to change it. `Ctrl+S` or **Save account** applies the
form; `Esc` or **Cancel** discards it. Secret keys and PINs are masked; when editing,
leave either blank to keep its existing value.

Account additions, edits, deletions, and manual counter changes remain pending until
you press `Ctrl+S` or quit and confirm **save**. Choosing **discard** drops pending
edits. Consuming an HOTP code persists its next counter immediately.
CLI subcommands remain available for scripting; run `cotp --help` for examples.

## TL;DR

```bash
# Display all the OTP codes in the interactive dashboard
cotp # select any code with arrow keys, press enter to copy into the clipboard, even in an SSH remote shell

# In the dashboard, press a to add or e to edit an account

# Add a new TOTP code from a BASE32 secret key
cotp add --label myaccount@gmail.com --issuer Google

# Add a new HOTP code with custom algorithm, digits and counter
cotp add --label example --type hotp --algorithm SHA256 --digits 8 --counter 10

# Edit the digits of the 4th OTP code
cotp edit --index 4 --digits 8

# List all the codes in JSON format passing password through stdin
echo "mysecretpassword" | cotp --password-stdin list --json

# Extract the uniquely matching OTP code with "google" issuer and copy it into the clipboard
cotp extract --issuer google --copy-clipboard

# Import an encrypted Aegis Database backup
cotp import --path my_db.json --aegis-encrypted

# Create an encrypted backup
cotp backup create ~/backups/cotp --keep 10

# Export secrets explicitly for migration
cotp export --plaintext --path migration.json
```

## Compatibility

cotp can generate both **TOTP** and **HOTP** codes, compliant with **rfc6238** and **rfc4226** specifications. Also, it is possible to customize settings like **HMAC algorithm** and **digits**, to provide compatibility to other two-factor authentication systems.

Latest releases also include support for **Steam**, **Yandex**, **MOTP** codes.

## Encryption

This program relies on only one database file encrypted
with [XChaCha20Poly1305](https://docs.rs/chacha20poly1305/latest/chacha20poly1305/) authenticated encryption
and [Argon2id](https://en.wikipedia.org/wiki/Argon2) for key derivation.

It also uses [AES-GCM](https://docs.rs/aes-gcm/latest/aes_gcm/) to import from encrypted Aegis backups.


## Cross Platform

cotp should be easily compiled on the most used platforms, but it is mostly tested on Linux, Windows and macOS.

# Install

## Arch Linux / Manjaro / Debian / Ubuntu / NixOS / others...

cotp is distributed in some of the mayor Linux distro repositories. Please check the repology badge at the top to know if your distribution already provides a package.
I will try to push to more repositories over time.

Otherwise you have other options as explained in the next paragraph.

## Other linux distributions and \*nix

Before beginning check that you have the required build dependencies to use the rust compiler.

You need to install the **libxcb-devel** dependency, needed for clipboard coping in X11:

- Debian based: `sudo apt install libxcb1-dev libx11-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev xclip`
- Fedora / RHEL based: `sudo dnf install libX11-devel`
- Void Linux `sudo xbps-install -S libxcb-devel`

## macOS

```
brew install cotp
```

## Windows

Building is supported with both `x86_64-pc-windows-gnu` and `x86_64-pc-windows-msvc` toolchains.

If you want to use `x86_64-pc-windows-msvc` you will need to install
the [Visual C++ 2019 Build Tools](https://visualstudio.microsoft.com/it/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16)

Once you have the rust toolchain installed just run `cargo install cotp`.

### Use the crates.io repository

Just type `cargo install cotp` and wait for the installation.

### Clone the GitHub repository and manually install

You can build cotp using these commands:

```
git clone https://github.com/replydev/cotp.git
cargo install --path cotp/
```

# Migration from other apps

cotp supports TOTP codes migration from various apps.
Some needs to be converted using simple python script you can find listed in the table below.

| App                                                                                                          | How to fetch backup                                                                                                                                                 | Needs conversion                                                          | cotp argument               |
|--------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------|-----------------------------|
| [andOTP](https://github.com/andOTP/andOTP)                                                                   | Make a backup using the app itself.                                                                                                                                 | No                                                                        | `--andotp`                  |
| [Aegis](https://github.com/beemdevelopment/Aegis)                                                            | Make a backup using the app itself.                                                                                                                                 | No                                                                        | `--aegis`                   |
| [Aegis](https://github.com/beemdevelopment/Aegis) (encrypted)                                                | Make an encrypted backup using the app itself.                                                                                                                      | No                                                                        | `--aegis-encrypted`         |
| [Authy](https://authy.com/)                                                                                  | Obtain `/data/data/com.authy.authy/shared_prefs/com.authy.storage.tokens.authenticator.xml` from your phone.                                                        | [Yes](https://github.com/replydev/cotp/blob/master/converters/authy.py)   | `--authy`                   |
| [Authy](https://authy.com/) (2nd method)                                                                     | Follow this guide: https://gist.github.com/gboudreau/94bb0c11a6209c82418d01a59d958c93.                                                                              | No                                                                        | `--authy-exported`          |
| [cotp](https://github.com/replydev/cotp)                                                                     | Export your database using `cotp export --plaintext`.                                                                                                                           | No                                                                        | `--cotp`                    |
| [FreeOTP](https://freeotp.github.io/)                                                                        | Obtain `/data/data/org.fedorahosted.freeotp/shared_prefs/tokens.xml` from your phone.                                                                               | [Yes](https://github.com/replydev/cotp/blob/master/converters/freeotp.py) | `--freeotp`                 |
| [FreeOTP+](https://github.com/helloworld1/FreeOTPPlus)                                                       | Make a backup using the app itself.                                                                                                                                 | No                                                                        | `--freeotp-plus`            |
| [Google Authenticator](https://play.google.com/store/apps/details?id=com.google.android.apps.authenticator2) | In the app open the menu, choose "Transfer accounts" > "Export accounts", select the accounts and let it generate the QR code(s). Scan each QR code with any scanner and save the resulting `otpauth-migration://...` URI(s), one per line, into a text file. | No                                                                        | `--google-authenticator`    |
| [Microsoft Authenticator](https://play.google.com/store/apps/details?id=com.azure.authenticator)             | Obtain `/data/data/com.azure.authenticator/databases/PhoneFactor` from your phone. Take also `PhoneFactor-wal`, `PhoneFactor-shm` if they exist in the same folder. | [Yes](https://github.com/replydev/cotp/blob/master/converters/mauth.py)   | `--microsoft-authenticator` |
| [OTP URI list](https://docs.yubico.com/yesdk/users-manual/application-oath/uri-string-format.html)           | Create a JSON file which contains a items property. It will contains a string array where each element is an OTP URI.                                               | No                                                                        | `--otp-uri`                 |

## How to convert

Once you got the correct files run the right python script located in the **converters/** folder in this source code.

**Example:**
`python authy.py path/to/database.xml converted.json`

It will convert the database in a json format readable by cotp.

To terminate the import:
`cotp import --authy --path path/to/converted_database.json`

# Configuration

Release builds use the platform data directory (`$XDG_DATA_HOME/cotp/db.cotp` on Linux),
with automatic migration from the legacy `$HOME/.cotp/db.cotp`. An existing `./db.cotp`
enables portable mode; debug builds always use that local path. Override the location
with `COTP_DB_PATH`, `--database-path`, or a named vault profile.
The first can be configured in shell configuration files, the second in package managers where the configuration of environment variables is not allowed, like Scoop.

These are examples of how to do this in bash:
```bash
$ COTP_DB_PATH=/home/user/.local/custom-folder/db.cotp cotp
```
or
```bash
$ cotp --database-path /home/user/.local/custom-folder/db.cotp
```

# Feature guides

- [Configuration, named profiles, shell completions, manuals, and headless builds](docs/configuration.md)
- [QR enrollment, repeatable imports, native migration formats, and encrypted Aegis exports](docs/imports.md)
- [Encrypted snapshot exchange and explicit conflict resolution](docs/exchange.md)
- [YubiKey OATH accounts and touch-required codes](docs/hardware.md)
- [Security model and release provenance](SECURITY.md), [vault format](docs/vault-format.md)

Run `cotp --help` and `cotp COMMAND --help` for the complete command reference.

# Contribution

I created this project for my own needs, but I would be happy if this little program is useful to someone else, and I
gratefully accept any pull requests.

## UI development

The repository pins Rust and Cargo in `rust-toolchain.toml`; rustup installs the selected
toolchain automatically. The interface uses Ratatui, Crossterm, and tui-input. Rendering
and interaction tests cover responsive layouts, keyboard and mouse actions, forms,
search, code timing, and masked fields.

To regenerate the documentation previews from the real widgets and demonstration data:

```bash
COTP_UI_PREVIEW=img/cotp-dashboard.svg COTP_FORM_PREVIEW=img/cotp-editor.svg cargo test interface::tests
```

To rebuild the animated demo with deterministic example accounts, install ImageMagick
(`magick`) and run:

```bash
python3 ci/render_ui_demo.py
```

The recording uses the actual dashboard widgets and keyboard/mouse handlers, with
synthetic accounts and masked secrets. Tutorial captions appear below the application.

### Encrypted backups and concurrent use

`cotp backup create ~/backups/cotp --keep 10` creates a private encrypted vault
backup and optionally retains the newest ten cotp backup files in that directory.
`cotp backup list ~/backups/cotp` lists backups without unlocking the vault.
Use `cotp backup verify FILE` to authenticate and validate a backup with its
original password. `cotp -d /path/to/vault backup restore FILE` restores a verified
backup; `--force` is required to replace an existing vault. The original backup
password remains in effect. Restoring an older backup can roll back HOTP counters;
resynchronize affected accounts with the service before consuming codes.
`--password-stdin` also works for backup verification and restoration.

Vault writes take a cross-process lock and reject stale snapshots, so an open
dashboard cannot silently overwrite another command's changes. If a save reports
that another process changed the vault, reopen it before retrying your edits.
Copying an HOTP code commits the incremented counter before releasing the code;
if the dashboard has unsaved edits, save those first. Displayed HOTP codes are
previews. A clipboard failure after successful persistence still consumes the
counter; this prevents a previously released code from being issued again.


The dashboard locks after 300 seconds without keyboard, mouse, or paste activity.
`cotp lock` requests a lock in every dashboard using the selected vault, without
requesting a password. Locking encrypts pending account edits and unfinished forms
in memory and drops the decrypted database and vault key. Enter the password to
resume; locking neither saves nor discards pending work.

Native clipboard copies expire after 30 seconds, provided the clipboard still
contains the copied value. A newer clipboard value is left intact. This comparison
is best effort: operating-system clipboards do not offer an atomic compare-and-clear.
SSH/OSC52 copies cannot be read back reliably and are never automatically cleared;
the dashboard reports this limitation. Set `idle_timeout_seconds` or
`clipboard_seconds` in the JSON configuration to change these defaults; `0`
disables the respective timer. Clipboard expiration continues after a CLI command
exits, using a helper process that receives the copied content through stdin.
