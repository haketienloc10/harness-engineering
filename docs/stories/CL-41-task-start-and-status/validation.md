# CL-41 Validation

Unit proof covers atomic rollback for an unknown story, primary-story linking,
required-story refusal, owner conflict for an active primary story, and
`in_progress → blocked → in_progress → abandoned`; a terminal task cannot
resume. It also covers a stored-manifest context acknowledgement, refusal of an
unlisted path, accepted workflow approval gate and rejected unknown gate.
It injects a stale checksum into a fixture DB, proves refresh reports a pending
delta without a write, then proves `--accept` commits the recomputed manifest.
It rejects a mismatched transition owner, then proves handoff changes the owner
before the recipient can perform lifecycle transitions.
It also proves a secondary story can be added and promoted to the sole primary.

Packaged black-box proof used a temporary `HARNESS_DB`: `init`, `story add`,
`task start --behavior-bearing yes`, `task block`, `task resume`, `task
abandon`, and `task status --json` all succeeded. The repository-local
`harness.db` was not written.

A separate packaged tiny task start followed by `task refresh --json` reported
identical checksums with `changed:false`, proving no-op refresh does not mutate.

Session/lease completion proof adds migration 010 plus a database insert guard
for partial identity. The 67-test Rust suite proves owner/session pairing,
bounded lease creation, session and worktree conflicts, primary-story
exclusivity after expiry, released leases on block, explicit renewal through
`task resume`, same-owner recovery into a new session only after expiry,
active-lease mismatch rejection, and source migrations `001..010`.

Source and installed-binary black-box runs used isolated temporary
`HARNESS_DB` files. They proved `task start --owner --session
--lease-seconds`, lease fields in JSON status, renewal of an `in_progress`
task, release on block, exit `2` for a partial identity, exit `8` for a session
conflict, atomic owner/session handoff, and resume under the recipient session.
The fixture databases were removed afterward.

Completion validation passed:

- `cargo fmt --check`
- `cargo test -p harness-cli` (67 passed)
- `cargo clippy -p harness-cli -- -D warnings`
- `_harness/bin/harness-cli workflow parity --json`
- `_harness/bin/harness-cli memory check --dry-run --json`
- `bash tests/installer_state_safety.sh`
- packaged `doctor --json` at canonical migrations `001..010`

Rollback retains the backup created before migration 010, restores the prior
installed binary and removes only the migration-010 contract changes. It must
not delete or rewrite task records manually.
