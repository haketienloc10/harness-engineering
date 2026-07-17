# 0020 Recoverable Task Closure

Date: 2026-07-14

## Status

Accepted

## Decision

`task finish` derives a deterministic closure nonce from the task ID and the
terminal capsule checksum (or explicit no-capsule disposition). Required
capsules are copied to a same-filesystem staged path, validated, then atomically
renamed while an SQLite `IMMEDIATE` transaction holds the task in `closing`.
The same transaction records the nonce and terminal capsule record.

If the process stops after rename and before commit, SQLite rolls back to the
active task and a repeated finish with the same inputs safely reuses the same
nonce and final capsule. A completed task accepts only its matching nonce and
capsule disposition; mismatches fail closed.

## Consequences

The source capsule must already be a valid repository-local artifact. Temporary
staging files are best-effort removed on pre-rename failures; doctor reports a
remaining staged closure file as unhealthy state. Existing applied migrations
remain unchanged; this decision adds migration `008-task-closure.sql`.
