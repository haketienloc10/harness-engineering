# CL-43 Validation

Unit proof creates a tiny owner-bound task, acknowledges context, records a
fresh structured proof and matching minimal trace, then completes it. A repeat
finish returns the existing completed result.

A second unit proof creates a normal behavior-bearing task, acknowledges every
stored must-read path, creates a valid capsule before proof capture, then
injects a terminal-SQL abort after staged rename. It proves SQLite rollback to
`in_progress`, preservation of the valid final capsule, a fresh structured
proof followed by retry with the same derived nonce, persisted
path/checksum/nonce and no leftover staged file.

Doctor unit proof inserts a completed required-capsule row with a missing file
and verifies `DB_UNHEALTHY` plus `TERMINAL_CAPSULE_INVALID`.

Another doctor proof leaves a `.closing-*.tmp` file under `docs/tasks/` and
verifies `DB_UNHEALTHY` plus `STAGED_CAPSULE_RECOVERY_REQUIRED`.

Packaged black-box proof on a temporary `HARNESS_DB`: tiny task start, context
acknowledgement, `proof run -- git --version`, a minimal trace for the task's
intake, then `task finish --outcome completed --friction none` returned
`completed`; status showed the matching fresh proof and acknowledged context.

Packaged normal proof created a temporary normal task/story, acknowledged its
three must-read paths, rendered a versioned capsule, ran proof after that Git
change, recorded a standard trace, and completed with `--capsule`; the final
capsule and fixture database were removed afterward.

CL-42 branch/output provenance, artifact freshness and derived-matrix contracts
are closed. On 2026-07-15 the final Phase 4 completion matrix passed against
those contracts, followed by installer state-safety, source and packaged
workflow parity, memory validation and strict doctor. CL-43 moved to
`completed` only after those gates succeeded.

## Final Phase 4 matrix

The table-driven repository matrix covers missing task root, explicit and
linked unresolved friction, missing approval, required capsule, missing story,
unmet context, missing/failing/stale proof and proof output, missing or
insufficient trace, docs/DB recovery mismatch, and owner conflict. Every
preflight row compares task closure fields and the complete capsule file set
before and after failure. The dedicated staging-failure row proves the same
invariant for a filesystem error.

The terminal-SQL abort test additionally snapshots the task and capsule set. It
proves rollback to the identical `in_progress` closure state, preservation of
the valid final capsule as the designed recovery artifact, cleanup of staged
files, and successful retry with the deterministic nonce. Repeated finish and
doctor orphan/terminal-capsule checks remain part of the workspace suite.

`crates/harness-cli/tests/phase4_failure_matrix.rs` runs the same human/JSON
contract as a source-binary black box and, after `install-harness-cli.sh`, as a
packaged `_harness/bin/harness-cli` black box. Each row asserts the exit code,
stable structured fields, non-empty remediation, presentation parity and no
task/capsule mutation.
