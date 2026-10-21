# Exchanging encrypted vaults

Use `cotp backup create DIRECTORY` to produce an encrypted snapshot. Transfer that
file using your preferred file transport, then authenticate it on the destination:

```sh
cotp backup verify remote.cotp-backup
cotp merge remote.cotp-backup --dry-run
cotp merge remote.cotp-backup --conflict keep-local
```

The source password may differ from the destination password. `--source-password-stdin`
reads the source password without placing it in process arguments. For automation,
provide the destination password separately through the global password FD option.
The source snapshot is authenticated and validated before merging anything. Changes
are written through the same locked, stale-write-protected transaction as other
commands. A rejected conflict or failed authentication writes nothing.

Account IDs identify accounts across copies; identical account material and names
also deduplicate independently imported entries. Conflicts abort by default. Choose
`--conflict keep-local` or `--conflict take-remote` explicitly. Aliases must remain
unique. Deletions are not propagated: an absent account does not prove deletion.
This is explicit snapshot merging, not a background cloud sync service.

HOTP accounts require `--hotp max`. Matching HOTP accounts retain the higher counter,
regardless of metadata conflict policy. Choose a single generating device for each
HOTP account: merging counters cannot prevent two disconnected devices from emitting
the same code. `backup restore --force` is full rollback, whereas `merge` preserves
higher counters for matching credentials.
