# Enrollment, migration, and plaintext export

Import PNG/JPEG QR screenshots, newline-separated OTP URIs, Google Authenticator migration URIs, or supported JSON backups directly. QR decoding runs locally and scans every QR code in the image. JSON format detection covers cotp, andOTP, Aegis, FreeOTP+, converted backups, Authy remote-debug exports, and OTP URI lists. Existing explicit format flags and `--path` remain available.

```sh
cotp import screenshot.png --dry-run
cotp import accounts.json --duplicates skip
cotp import accounts.json --duplicates replace
cotp import accounts.json --duplicates keep-both
cotp import - < otp-uris.txt
cotp import --aegis --path aegis.json
```

Imports display account names and parameters on stderr before saving. Interactive imports request confirmation; `--yes` accepts the preview. A dry run never changes existing vault contents. Imports reject the entire batch if any entry is invalid unless `--skip-invalid` is supplied. Errors identify source entries without printing secrets. PNG/JPEG inputs are limited to 32 MiB and bounded decoded image dimensions/memory.

Repeated imports skip duplicate credentials by default. Identity comprises the normalized secret, OTP scheme, algorithm, digit count, period, and PIN; account display names do not affect identity. Replacement preserves the greater HOTP counter, so an older backup cannot roll the counter backwards. `keep-both` deliberately adds duplicate credentials.

Plaintext exports now require explicit acknowledgment:

```sh
cotp export --plaintext --path migration.json
cotp export --plaintext --otp-uri --path otp-uris.json
```

The default filename is `exported.plaintext.json`. Exports contain authentication secrets. Files are created privately and atomically, and existing paths are never overwritten. Use the encrypted backup workflow for routine backups.

## Native migrations

These formats are detected automatically; explicit flags are available when needed:

| Application | Flag | Supported input |
| --- | --- | --- |
| 2FAS | `--2fas` | Plaintext schema 4 `.2fas` JSON with `services[].secret` and `services[].otp`; compatible earlier services containers are accepted |
| Ente Auth | `--ente` | Decrypted text export with one `otpauth://` URI per line |
| Bitwarden | `--bitwarden` | Unencrypted Authenticator or Password Manager JSON, `items[].login.totp` as an OTP URI, bare base32 seed, or `steam://` seed |
| 1Password | `--onepassword` | `.1pux` ZIP containing `export.data`, or extracted `export.data` JSON; imports every `details.sections[].fields[].value.totp` |

2FAS group names become tags and entry positions are retained. Bitwarden favorites and 1Password favorites/tags are retained. Entries without OTP data in password manager exports are ignored. Passwords, notes, attachments, and unrelated fields are never imported. 1PUX attachments are not extracted. The decompressed `export.data` limit is 32 MiB. Encrypted 2FAS, Ente, Bitwarden exports and 1Password CSV/1PIF are not supported; create a plaintext export in the source application first.

```sh
cotp import backup.2fas --dry-run
cotp import ente-auth.txt --ente --yes
cotp import bitwarden.json --yes
cotp import account.1pux --yes
```

Source schemas: [2FAS BackupContent](https://github.com/twofas/2fas-android/blob/main/data/services/src/main/java/com/twofasapp/data/services/domain/BackupContent.kt), [2FAS BackupService](https://github.com/twofas/2fas-android/blob/main/data/services/src/main/java/com/twofasapp/data/services/domain/BackupService.kt), [Ente export documentation](https://ente.com/help/auth/migration/export), [Bitwarden Authenticator export examples](https://bitwarden.com/help/authenticator-import-export/), and [1Password 1PUX specification](https://support.1password.com/1pux-format/). The fixtures in `test_samples/migrations` are synthetic examples built from these structures, containing only dummy credentials.

## Selected and encrypted exports

Export selections combine with AND: repeated `--id` values select any of those IDs, `--tag` matches a tag case insensitively, and `--query` matches a case insensitive substring in issuer, label, alias, or tags. Empty selections and unknown IDs are errors.

```sh
cotp export --plaintext --query example --tag work --path selected.json
cotp export --plaintext --id ACCOUNT_ID --otp-uri --path selected-uris.json
cotp export --aegis-encrypted --path aegis.json
cotp export --aegis-encrypted --export-password-file /private/export-password --path aegis.json
cotp import aegis.json --import-password-file /private/export-password --yes
```

`--aegis` produces plaintext Aegis JSON and requires `--plaintext`. `--aegis-encrypted` uses a separate password, obtained through a verified terminal prompt or `--export-password-file`. Import supports the corresponding `--import-password-file`; passwords are never passed as argument values. Password files have a 4096 byte limit and trailing newlines are removed. Aegis export supports TOTP, HOTP, and Steam, including algorithms, digits, periods, HOTP counters, favorites, and tags as Aegis groups. Other OTP types cause an error rather than being silently dropped. cotp-only IDs, aliases, and ordering are preserved by cotp JSON exports.

The encrypted Aegis container uses a random 256-bit master key, scrypt (`N=32768, r=8, p=1`) password slot, and AES-256-GCM, following [Aegis password slots](https://github.com/beemdevelopment/Aegis/blob/master/app/src/main/java/com/beemdevelopment/aegis/vault/slots/PasswordSlot.java) and [entry serialization](https://github.com/beemdevelopment/Aegis/blob/master/app/src/main/java/com/beemdevelopment/aegis/vault/VaultEntry.java). Exports are created privately without overwriting existing files.
