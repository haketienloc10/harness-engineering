# CL-11 Idempotent Ensure and Safe Migration

Status: completed

Reopened 2026-07-15 after the retained repository DB became behind source:
both the packaged CLI and the source CLI reported
`sqlite error: no such table: migration_history` instead of migrating schema
`008` to `009`. The DB was not edited manually. Completion again requires a
backup-first migration from this legacy `schema_version`-only state, followed
by a healthy doctor result and an idempotent second ensure.

Further recovery inspection found that the restored `008` DB had no canonical
`task` table required by main migration `006-command-first-foundation.sql`.
It was therefore foreign/incomplete, not a compatible legacy main-lineage DB.
The reviewed direction selected canonical rebuild/import recovery (ADR 0010
Case B). The foreign DB was checkpointed and retained as a backup, then a
validated canonical rebuild atomically replaced the active DB.

## Contract

Compatibility `init` and `migrate` delegate to one `ensure` path. It accepts
only a missing DB, a healthy DB, or a DB demonstrably behind the canonical
source; it rejects unversioned, unhealthy, ahead, and foreign-lineage state
without a write.

## Implemented So Far

- New DBs are built at a same-filesystem temporary path and renamed only after
  all source migrations and checksum history records commit.
- Existing behind DBs are copied to `harness.db.backups/` before a SQLite
  `IMMEDIATE` transaction applies pending migrations.
- `migration_history` records source filename, SHA-256, CLI version, and Git
  commit after migration `006` creates the table.
- `ensure` reruns doctor after an upgrade and rejects a non-`HEALTHY` result.
- Ahead DBs are rejected before backup or write; healthy DBs are idempotent.

## Validation Evidence

2026-07-14:

- `cargo test -p harness-cli` — 45 passed, including legacy v1 backup/migrate,
  current-db idempotency, ahead-db no-write/no-backup, transaction rollback,
  backup restore, and retention of the newest five backups.
- `cargo fmt --all -- --check`, `cargo clippy -p harness-cli -- -D warnings`,
  and `git diff --check` — passed.
- `cargo run -q -p harness-cli -- migrate` against the retained ahead DB exits
  `3` with no write, as required for unsafe durable state.

2026-07-15 regression audit:

- `doctor --json` reports `DB_BEHIND_SOURCE`, with source migrations `001..009`
  and retained DB migrations `001..008`.
- `_harness/bin/harness-cli migrate` and
  `cargo run -q -p harness-cli -- migrate` both exit non-zero with
  `sqlite error: no such table: migration_history`.
- No SQL workaround was applied. The retained DB remains at migration `008`.

2026-07-15 recovery proof:

- `memory rebuild --dry-run --json` produced a `HEALTHY` canonical candidate
  with 31 projected artifacts and logical digest
  `ec9c3d93d01c86ad56033fb61563c9dbc0c476ccda25cd0744e5695389bfd7c8`.
- `memory rebuild --apply --recover-foreign --json` backed up the quarantined
  DB as `harness.db.backups/rebuild-607656.db` and atomically installed the
  validated candidate.
- Packaged and source `doctor --json` report `HEALTHY`, main lineage and source
  plus DB versions `001..009`; integrity and foreign-key checks pass.
- The rebuilt task table is empty and has no link to excluded foreign stories.
- Missing main-lineage metadata at version `006+` now fails closed; ensure no
  longer synthesizes lineage or checksum history for an unproven database.

## Handoff

- Backup paths include a nanosecond timestamp, pre-migration version, lineage,
  and source checksum prefix; WAL/SHM sidecars are retained with the same base
  name and cleaned with expired backups.
- CL-41 will invoke `ensure` through `task start`. Legacy `init` and `migrate`
  already delegate to it and preserve their output shape.

## Rollback

The ensure path preserves a pre-migration backup. Revert source changes and
restore a reviewed canonical backup only; the quarantined foreign backup is
recovery evidence and must not become the active main-lineage DB again.
