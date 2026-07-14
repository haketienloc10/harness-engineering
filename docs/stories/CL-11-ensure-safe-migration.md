# CL-11 Idempotent Ensure and Safe Migration

Status: completed

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

## Handoff

- Backup paths include a nanosecond timestamp, pre-migration version, lineage,
  and source checksum prefix; WAL/SHM sidecars are retained with the same base
  name and cleaned with expired backups.
- CL-41 will invoke `ensure` through `task start`. Legacy `init` and `migrate`
  already delegate to it and preserve their output shape.

## Rollback

The ensure path preserves a pre-migration backup. Revert source changes and
restore the matching `.bak` plus any WAL/SHM sidecars only through a reviewed
recovery procedure; never overwrite an ahead/foreign DB automatically.
