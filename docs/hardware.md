# YubiKey OATH backend

Install YubiKey Manager (`ykman`) and enable the key's OATH application. cotp invokes
its CLI directly without a shell, keeping the OTP secret on the device. Password
and touch prompts are handled by ykman. Multiple connected keys can be selected
with the global `--device SERIAL` option.

```sh
cotp --backend yubikey list --json
cotp hardware code 'GitHub:me@example.com'
cotp hardware code 'GitHub:me@example.com' --copy --quiet
cotp --device 12345678 hardware add 'me@example.com' --issuer GitHub --touch
cotp hardware delete 'GitHub:me@example.com'
```

Enrollment prompts for the secret instead of putting it in process arguments or
shell history. Enrollment and deletion require an interactive terminal. Queries
must match exactly one account; an exact full name takes precedence over substring
matches. `ykman --single` enforces single matching again on generation. HOTP counters
advance on the key, even if output delivery later fails; cotp never retries a code
generation automatically. Listing does not generate codes or require touch.

Device accounts use their full OATH names as IDs. Local vault UUIDs, tags and aliases
are not copied onto the device. Hardware JSON reports `expires_at: null` because the
CLI adapter does not know each credential's period. No local vault is opened for
hardware commands. Device/password errors propagate as failures rather than falling
back to a software copy of a secret. Without interactive input, ykman's stdin is
closed so a password prompt fails instead of hanging.

The backend is tested with a deterministic subprocess fixture; physical-device and
touch behavior requires a real key and is not exercised by the regular test suite.
Reference: https://docs.yubico.com/software/yubikey/tools/ykman/OATH_Commands.html
