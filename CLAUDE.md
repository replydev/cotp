# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

cotp is a command-line TOTP/HOTP authenticator written in Rust (edition 2024). It stores all secrets in a single encrypted database file and provides both a subcommand CLI and an interactive terminal dashboard (TUI). The crate forbids `unsafe_code` (`#![forbid(unsafe_code)]` in `main.rs`).

## Common commands

```bash
cargo build                          # debug build (version suffixed -DEBUG-<commit>)
cargo build --release                # optimized release build
cargo test --locked                  # run all tests (unit + integration)
cargo test --locked --release        # tests in release mode (CI runs both)
cargo test <name>                    # run a single test by substring match
cargo fmt --all -- --check           # formatting check (CI gate)
cargo clippy -- -D warnings          # lint; warnings are errors (CI gate)
cargo clippy --all-targets -- -D warnings  # also lints tests; kept clean too
```

CI (`.github/workflows/build.yml`) requires `cargo fmt`, `cargo clippy -D warnings`, and `cargo test` (debug + release) to pass across Linux/macOS/Windows. On Linux, building requires xcb dev libraries for clipboard support (see README "Other linux distributions").

**Before finishing any work you MUST run clippy exactly as CI does — `cargo clippy -- -D warnings` — and fix every issue it reports.** CI treats warnings as errors, so any lint left unaddressed will fail the pipeline. Do not consider a task complete until this command exits cleanly.

The Python converters have their own tests: `cd converters && python -m pytest` (or run `test_converters.py`), exercised against fixtures in `example_databases/`.

## Commit conventions & versioning

Commits **must** follow the [Conventional Commits](https://www.conventionalcommits.org/) / Angular convention: a `type(scope): subject` header where `type` is one of `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, or `revert`. The scope is optional (e.g. `feat(import): ...`).

Releases are fully automated by **semantic-release** on every push to `main` (`.releaserc`, `.github/workflows/release.yml`), and the commit types are what drive the version bump — there is no manual editing of the version in `Cargo.toml`:

- `fix:` → **patch** release (e.g. 1.9.10 → 1.9.11)
- `feat:` → **minor** release (e.g. 1.9.10 → 1.10.0)
- a commit with a `BREAKING CHANGE:` footer (or `!` after the type, e.g. `feat!:`) → **major** release
- other types (`docs`, `chore`, `ci`, …) → no release

commit-analyzer computes the next version, `ci/write_cargo_version.sh` writes it into `Cargo.toml`/`Cargo.lock`, `CHANGELOG.md` is regenerated, and the Deploy workflow is triggered. **Choose the commit type deliberately — it directly determines the published version.** Keep one logical change per commit so the release notes stay meaningful.

## Runtime data flow

`main.rs` drives a fixed lifecycle for every invocation:

1. `init()` — resolves the DB path, and either initializes a new encrypted database on first run (prompting for a password) or loads the existing one via `storage::`. Returns `ReadResult = (OTPDatabase, key, salt)` (defined in `storage/mod.rs`).
2. `args_parser()` (`arguments/mod.rs`) — if a subcommand was given, dispatches to it; otherwise launches the interactive `dashboard()` (TUI). Each subcommand consumes the `OTPDatabase` and returns a (possibly modified) one. Exit codes: 1 for init/save errors, 2 for subcommand errors.
3. Back in `main()`, if `database.is_modified()` the database is re-encrypted and written to disk via `storage::save()`. The derived `key` is zeroized before exit.

The `OTPDatabase` is passed by value through the command layer; mutations set a private dirty flag (via `mark_modified()`/`clear_modified()`) that gates the final save — `mut_element()` deliberately does NOT mark, so callers mark only on real changes (no-op edits skip the rewrite). `storage::save()` clears the flag only after a successful write; this is also what makes `passwd` safe (it saves itself with a key from the new password, and the cleared flag stops `main()` from saving again with the old key). Secrets and keys use `zeroize` throughout — preserve zeroization when touching password/key handling.

## Key modules (`src/`)

- **`arguments/`** — Clap subcommands (`add`, `edit`, `list`, `delete`, `import`, `export`, `extract`, `passwd`). Each implements the `SubcommandExecutor` trait (`fn run_command(self, db: OTPDatabase) -> Result<OTPDatabase>`), wired together with `enum_dispatch` on the `CotpSubcommands` enum. To add a subcommand: create the module, define an `Args` struct, implement `SubcommandExecutor`, and add a variant to `CotpSubcommands`. The mutually exclusive import/export format flags map to exhaustive internal enums (`ImportFormat` in `import.rs`, `ExportKind` in `export.rs`) — add new formats there. `extract` matches issuer/label with a hand-rolled `*`/`?` wildcard matcher (no regex/glob crate).
- **`otp/`** — core domain. `otp_element.rs` holds `OTPElement` and `OTPDatabase`; the database is pure domain data (element accessors, dirty flag, sort) — persistence lives in `storage/`. `algorithms/` has one generator per scheme (`totp`, `hotp`, `motp`, `steam`, `yandex`); Yandex always uses HMAC-SHA256 regardless of the element's stored algorithm. `otp_type.rs` / `otp_algorithm.rs` are the enums, with `TryFrom<&str>` impls that reject unknown strings instead of defaulting; `from_otp_uri.rs` parses `otpauth://` URIs.
- **`storage/`** — persistence layer. Load: `get_elements_from_input`/`get_elements_from_stdin` → `read_from_file` (password prompt, decrypt, legacy-v1 fallback). Save: `save(db, key, salt, path)` / `save_with_pw(db, password, path)` — runs `migrate()` on every save, encrypts, writes with 0600 permissions on unix, zeroizes the plaintext JSON, and clears the dirty flag only after a successful write.
- **`crypto/`** — `cryptography.rs` does Argon2id key derivation (config constants at top of file) + XChaCha20Poly1305 authenticated encryption; also AES-GCM for decrypting Aegis encrypted backups. `encrypted_database.rs` is the on-disk envelope.
- **`importers/`** — one module per source app. `importer.rs::import_from_path::<T>()` is the generic entry point: `T` must be `Deserialize + TryInto<Vec<OTPElement>>`. Import selection happens in `arguments/import.rs`. Some sources (Authy, Microsoft Authenticator, FreeOTP) are pre-converted by Python scripts to `ConvertedJsonList` first (see below); others deserialize natively. Google Authenticator is handled natively by `google_authenticator.rs`, which parses `otpauth-migration://` export URIs (base64 protobuf `MigrationPayload`, decoded with `prost` using hand-declared message structs — no `.proto`/`protoc` build step).
- **`exporters/`** — `andotp`, `freeotp_plus`, `otp_uri`. `do_export::<T: Serialize>()` is the shared writer (0600 permissions on unix).
- **`interface/`** — the ratatui/crossterm TUI. `app.rs` holds mutable `App` state (including the cached rendered QR code); `ui.rs` owns the terminal lifecycle (`Tui`) and all rendering, as free functions taking `&mut App`; `event.rs` is the input event loop (250ms tick); `handlers/` route key events by focus (`main_window`, `popup`, `search_bar`). The dashboard runs on `io::stderr()` so stdout stays clean for piping.

## Dependency notes

Errors use plain `eyre` (not color-eyre). All base32/hex/base64 codecs go through `data-encoding` (no `hex`/`base64` crates). `ratatui` and `qrcode` are built with trimmed feature sets, and `url`'s IDNA backend is pinned to the small unicode-rs `idna_adapter` — keep this binary-size budget in mind when adding dependencies.

## Database format & migrations

- Default path resolution (`path.rs`): `--database-path` arg > `COTP_DB_PATH` env > `./db.cotp` (portable / debug builds always) > `$XDG_DATA_HOME/cotp/db.cotp` (auto-migrated from legacy `$HOME/.cotp/db.cotp` if present). The path is a `OnceLock` set once at startup.
- `CURRENT_DATABASE_VERSION` (in `otp_element.rs`) is the schema version. Legacy v1 was a bare `Vec<OTPElement>`; `storage::read_from_file` falls back to parsing that and converts via `From<Vec<OTPElement>>`. Schema upgrades go in `otp/migrations/mod.rs` — add a `Migration { to_version, migration_function }` entry to `MIGRATIONS_LIST`; `migrate()` runs on every save (from `storage::save`).

## Python converters (`converters/`)

Some app backups are binary/XML and need pre-conversion into cotp's JSON (`ConvertedJsonList`) before `cotp import`. Scripts: `authy.py`, `gauth.py`, `mauth.py`, `freeotp.py`. Usage: `python <script>.py path/to/backup converted.json`, then `cotp import --<app> --path converted.json`. Fixtures live in `example_databases/`; keep `test_converters.py` in sync when changing output shape.

## Testing notes

- `tests/` holds integration tests using `assert_cmd`/`assert_fs`/`predicates` that invoke the built `cotp` binary with a throwaway database. `test_samples/` holds import/export fixtures.
- The password from stdin (`--password-stdin`) and `--database-path` flags are the primary levers for scripting cotp non-interactively in tests.
