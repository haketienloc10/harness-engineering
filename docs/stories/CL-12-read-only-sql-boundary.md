# CL-12 Read-Only SQL Boundary

Status: completed

## Contract

`query sql` is diagnostic-only: it opens SQLite read-only, accepts one
read-only `SELECT`, `WITH`, or allowlisted diagnostic `PRAGMA` statement, and
rejects all normal write/DDL/attach/vacuum paths before execution.

## Validation Evidence

2026-07-14:

- `cargo test -p harness-cli` — 46 passed. The mutation corpus rejects
  `INSERT`, `UPDATE`, `DELETE`, DDL, `ATTACH`, writable `PRAGMA`, `VACUUM`, and
  multi-statement input while the DB checksum remains unchanged.
- Read-only `SELECT`, `WITH`, and `PRAGMA table_info` cases pass.
- `cargo fmt --all -- --check`, `cargo clippy -p harness-cli -- -D warnings`,
  and `git diff --check` — passed.

## Rollback

Remove the query boundary implementation from source only. It performs no DB
write and leaves existing databases unchanged.
