# CLI standard feature integration

The implementation is assembled on `feature/cli-standard`; `main` is not modified.
Feature branches and their isolated worktrees are retained for review. Worktrees
are siblings of the integration checkout, named `standard-<area>`.

| Feature branch | Delivered behavior |
| --- | --- |
| `feature/standard-accounts` | Persistent IDs, aliases, one-based selectors, ambiguity errors, metadata-preserving schema migration |
| `feature/standard-storage` | Cross-process locking, stale-write checks, durable HOTP consumption, encrypted backup create/list/verify/restore and retention |
| `feature/standard-imports` | Local PNG/JPEG QR decoding, OTP/migration URI input, automatic format detection, dry runs, duplicate policies, per-entry validation, explicit private plaintext exports |
| `feature/standard-reliability` | Full HOTP/TOTP RFC vectors, malformed-input property tests, cryptographic envelope validation, security/format documentation, release checksums and signed provenance |
| `feature/standard-shell` | JSON settings, named vault profiles, shell completions, generated manual pages, build without native clipboard dependencies |
| `feature/standard-cli` | Single-account `code`, minimum validity, watch, JSON expiry, metadata-only listing, noninteractive setup, password FDs, clean output and documented exit codes |
| `feature/standard-migrations` | 2FAS/Ente/Bitwarden/1Password imports, 1PUX archives, selective export, encrypted Aegis export, fixture-based interoperability tests |
| `feature/standard-session` | Inactivity/manual/cross-process lock, encrypted pending-edit restoration, explicit save shortcut, native clipboard expiration with ownership comparison |
| `feature/standard-hardware` | YubiKey OATH listing/code/enrollment/deletion through ykman, device selection, single-match generation and touch-capable enrollment |
| `feature/standard-exchange` | Authenticated encrypted snapshot merging, dry runs, explicit conflict policy, HOTP max-counter policy, no silent deletion propagation |
| `feature/standard-organization` | Tags, favorites, custom ordering, shared CLI/TUI filtering and account metadata |
| `feature/standard-keychain` | Opt-in native OS credential storage, authenticated remember/forget/status, vault-scoped credentials and explicit-password precedence |
| `feature/standard-review` | Integration review fixes for plaintext/key lifetimes and malformed backup handling |
| `feature/standard-docs` | Consolidated user guides and headless CI checks |
| `fix/standard-converter-tests` | Import-safe Python converter entry points |

## Integration validation

The merged implementation passed these checks on Linux aarch64:

```sh
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test --locked --quiet
cargo test --locked --release --quiet
cargo clippy --locked --no-default-features -- -D warnings
cargo test --locked --no-default-features --quiet
uv run --with pytest python -m pytest -q converters
```

Each Rust configuration passed 120 unit tests and 24 integration tests. The
cross-process locking helper is marked ignored in the general suite and invoked
explicitly by its parent test. All four Python converter tests passed. macOS,
Windows, physical hardware, native credential stores, and release attestation
were not exercised locally.

## Deliberate boundaries

- Snapshot exchange is local and explicit. No hosted service or background network
  synchronization is introduced. HOTP use must be assigned to a single generating
  device; merging counters does not make disconnected concurrent generation safe.
- Native clipboard clearing is best effort. OSC52 cannot safely read back clipboard
  ownership, so SSH copies are not automatically erased.
- YubiKey support requires `ykman`; hardware timing/watch/preview options are rejected
  when their semantics cannot be guaranteed. Hardware JSON expiry is unknown.
- Keychain integration is opt-in. Session reauthentication still requires the vault
  password; locking does not delete a remembered OS credential.
- New authenticator imports accept the documented export formats. Unsupported
  proprietary encrypted formats fail with instructions to export supported input.
- Native device/credential stores and GitHub release attestation require their host
  environments. Automated tests use synthetic vaults and backend fixtures; no user
  credentials are read or written during verification.

See the linked guides in README for commands, compatibility changes, and configuration.
