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
