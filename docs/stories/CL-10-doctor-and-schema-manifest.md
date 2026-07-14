# CL-10 Doctor and Schema Manifest

Status: completed

## Contract

`harness-cli doctor` inspects source migration lineage and the local SQLite
database without creating, migrating, repairing, or otherwise mutating durable
state. Its machine-readable result is the shared preflight contract for later
operational commands.

## Acceptance Criteria

- Source SQL files are checked against the versioned main-lineage manifest,
  including filename, continuity, duplicate version, and SHA-256 checks.
- Doctor uses a read-only SQLite connection and reports missing, unversioned,
  corrupt, ahead, behind, lineage-mismatched, and checksum-mismatched DBs.
- JSON output has stable `ok`, `code`, `message`, `details`, and
  `remediation` fields; unsafe durable state exits with code `3`.
- A checksum- and lineage-verified latest DB reports `HEALTHY`.
- Doctor resolves a unique repository root, reports platform, required payload
  paths and managed DB ignores, and rejects unsupported workflow policy majors.

## Validation Evidence

2026-07-14:

- `cargo test -p harness-cli` — 40 passed, including missing, corrupt,
  unversioned, v1-behind, ahead non-mutating, foreign-key-invalid,
  checksum-invalid, foreign-lineage, source-gap/duplicate, and
  workflow/payload-invalid databases.
- `cargo clippy -p harness-cli -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `bash -n install.sh` and `bash -n install-harness-cli.sh` — passed.
- `git diff --check` — passed.
- `cargo run -q -p harness-cli -- doctor --json` against the preserved local
  DB returned `DB_AHEAD_OF_SOURCE`, source `001..006`, DB `001..008`, and exit
  code `3`; it made no DB write. It also reports the currently missing
  `AGENTS.md` and `.harness-id` as required-payload findings rather than
  creating them. The result includes platform, repository ID, canonical
  worktree, branch, and commit provenance.

## Rollback

Remove the doctor command and manifest only from source. It never mutates the
existing DB, and the retained CL-00 recovery copy remains untouched.

## Handoff

- `HarnessService::preflight()` is the shared, read-only service boundary;
  `doctor` renders the same result.
- CL-11 owns wiring that boundary into state-changing `ensure`/compatibility
  `init`, including backup-first transactional migration. It must preserve the
  CL-10 fail-closed behavior for ahead and foreign-lineage databases.
