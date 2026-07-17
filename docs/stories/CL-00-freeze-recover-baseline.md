# CL-00 Freeze and Recover Current Durable Truth

Status: completed

The required review is resolved by accepted decision 0010 and the approved
2026-07-14 CL-22-unblock preparation recorded below.

## Contract

Preserve the current local `harness.db` without rewriting it, produce a
verified recovery copy, and make the schema-lineage decision evidence explicit
before any command-first schema or CLI work begins.

## Acceptance Criteria

- The original `harness.db` is not modified or deleted.
- A recovery copy exists outside the normal database path and has the same
  SHA-256 checksum as the source database.
- The recovery copy passes SQLite integrity and foreign-key checks.
- Source and database migration inventories, including provenance for DB-only
  migrations and artifacts, are recorded.
- The main-lineage rebuild input set and unresolved human decisions are listed.

## Validation Evidence

Recorded on 2026-07-14 from worktree commit
`ae580d7446b6d37a578fcf386f98f8612fe6cffe` (`feature-rework`):

- Recovery copy: `.harness-backup/cl-00-20260714T000000+0700/harness.db`
- SHA-256 for source and copy:
  `e5664491266d62c5747ec111a90f64151eb64fa1ea3f4e7da5fbc1112c78e6ff`
- No `harness.db-wal` or `harness.db-shm` was present at capture time.
- The copied DB returned `ok` for `PRAGMA integrity_check` and no rows for
  `PRAGMA foreign_key_check` when opened through `HARNESS_DB`.
- Canonical source on the baseline commit contains migrations `001..005`.
- The local DB records versions `001..008`; `006..008` are present only in
  `SYMPHONY` commit `d9b870741b2ae9d7f586de1dc2f998a78f753f96`:
  - `006-changeset-applied.sql` —
    `848e75302f44ad432b16a67587200f1a63e96781f4536d0c27ec3942a871f0f2`
  - `007-story-dependencies.sql` —
    `e8da65d1a3f52a1c5f8b854304b338ce35ccf5e1f28c4c59d2b399b3f977f906`
  - `008-story-hierarchy.sql` —
    `b946511bad3be81aa3f21db935ca3216b05fb9d8b562417b70752523223710a3`
- DB-only story `US-004` and decision
  `0010-experimental-sync-with-stable-layout` are also present in that
  `SYMPHONY` commit. They are not canonical `main` artifacts.

## Main Rebuild Input Set

Until a human selects Case A or Case B, the main-lineage input set is:

- Git-tracked source migrations `001..005` at the baseline commit.
- Git-tracked stories and decisions in `docs/stories/` and `docs/decisions/`
  at the baseline commit.
- The recovery copy above, retained only as evidence/recovery input and never
  treated as a healthy main-lineage database.

## Required Human Decisions

1. Case B is accepted in decision
   `0010-main-schema-lineage-without-symphony`: quarantine/rebuild from main
   `001..005`.
2. `US-004` and the `SYMPHONY` experimental decision are foreign-lineage
   records and excluded from the main rebuild by decision 0010.
3. Approve the remaining CL-01 ADR directions before CL-10 begins.
4. Resolved by the approved 2026-07-14 CL-22-unblock preparation: tracked
   repository identity `59342e22-7493-400b-99b6-985c38310d85` was introduced
   without modifying the retained recovery database.

## Rollback

Delete only `.harness-backup/cl-00-20260714T000000+0700/` if its retention is
not approved. Do not alter the original `harness.db`.
