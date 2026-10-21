# Security

Report suspected vulnerabilities privately to commoncargo@tutanota.com. Include the
cotp version, operating system, reproduction using synthetic data, and expected
behavior. Never attach a real vault, password, OTP secret, URI, or backup code.

## Threat model

The encrypted vault protects secrets at rest when the password is unknown. It does
not protect an unlocked session from malware running as the same user, a compromised
terminal, clipboard history, screen capture, or privileged memory inspection. OTP
codes and QR/URI exports are sensitive. The application makes no claim of a completed
independent security audit.

Password-based operation is offline. Optional integrations have their own trust
boundaries and require explicit configuration. Back up the encrypted vault before
upgrading; losing the password and all usable backups loses access to its secrets.

See [vault format](docs/vault-format.md) for the encryption and versioning contract.
The test suite covers all RFC 4226 Appendix D and RFC 6238 Appendix B vectors and
property-based malformed-envelope inputs. Run `cargo test malformed_tests` for the
latter; `PROPTEST_CASES=10000` increases the case count. Never run experimental fuzzing against a real vault.

## Verify release provenance

Releases produced by the updated Deploy workflow include SHA256SUMS and GitHub
artifact attestations signed using the workflow's OIDC identity. Download an archive
and check it with:

```sh
sha256sum --check SHA256SUMS --ignore-missing
gh attestation verify cotp-VERSION-PLATFORM.tar.xz --repo replydev/cotp
```

Attestations bind the artifact digest to the repository/workflow that built it;
they do not constitute a security audit. Older releases may not have attestations.
