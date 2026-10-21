# Vault format

The database is a UTF-8 JSON envelope with `version`, `nonce`, `salt`, and `cipher`.
Envelope version 1 uses standard padded Base64 for the binary fields:

- `salt`: 16 random bytes, freshly generated on password change.
- `nonce`: 24 random bytes, freshly generated on every encryption.
- `cipher`: XChaCha20-Poly1305 ciphertext with its authentication tag.

The 32-byte encryption key is derived using Argon2id version 0x13 with 32768 KiB
memory, four iterations and four lanes. The encrypted payload is JSON containing a
separate schema `version` and an `elements` array. Historical schema 1 may instead
be a bare array. Schema migrations preserve OTP secrets and parameters. The
cryptographic envelope version is separate from the payload schema version; do not
change KDF parameters without a new envelope version and migration support.

Payload schema 3 assigns a persistent random 128-bit hexadecimal ID to each account.
Accounts also carry an optional alias, tags, favorite status, and an optional custom
sort order. Legacy migration assigns missing IDs and repairs collisions.

An account stores its issuer, label, Base32 secret, type, HMAC algorithm, digit
count, period, optional HOTP counter and optional scheme-specific PIN. Never log
the payload. Plaintext portability exports must be treated like passwords.

Unknown envelope versions, malformed Base64 and invalid nonce/salt lengths are
rejected. Authentication failure can mean a wrong password or damaged ciphertext;
no plaintext is returned in either case. The envelope metadata is not additional
authenticated data in version 1; its version and lengths are checked before use.

Writes use a temporary file in the destination directory, sync its content, then
atomically replace the destination. On Unix the directory is synced as well.
Atomic replacement prevents partial-file writes; transaction conflict handling is
a separate layer. Filesystem durability still depends on the filesystem and hardware.
