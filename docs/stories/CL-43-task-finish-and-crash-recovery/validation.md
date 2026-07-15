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

Required before completion: close CL-41 session/lease identity and CL-42
branch/output provenance plus artifact-scoped freshness, then rerun the Phase 4
completion matrix against those final contracts. The implementation proof
above is retained, but it does not close dependencies that remain
`in_progress`.
