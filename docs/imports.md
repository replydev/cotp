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
