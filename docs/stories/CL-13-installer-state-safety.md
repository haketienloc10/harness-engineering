# CL-13 Installer State Safety

Status: completed

## Contract

The installer updates only managed Harness payload. It preserves target product
files and local DB state, owns a delimited `.gitignore` block, creates a stable
tracked repository identity only when absent, and reports unsupported binary
platforms with remediation.

## Implemented So Far

- `install.sh` installs/replaces only a `HARNESS local-state` `.gitignore`
  block for DB/WAL/SHM, backups, evidence, and staged capsules; other user
  entries are preserved.
- It creates `.harness-id` once via `uuidgen`/`openssl`, preserves it on
  reinstall, and tells the user to commit it.
- It rejects unsupported platform/architecture before downloading/copying the
  Linux x86_64 binary.
- Completion output explicitly states that payload changed but `harness.db`
  was not touched and that ensure is pending.

## Validation Evidence

2026-07-14:

- `bash -n install.sh` and `bash -n install-harness-cli.sh` — passed.
- `bash tests/installer_state_safety.sh` — passed: fresh install, reinstall,
  existing product doc and DB checksum preservation, stable `.harness-id`, one
  managed ignore block, and unsupported-platform remediation.
- `cargo test -p harness-cli` — 46 passed; fmt, clippy and `git diff --check`
  passed.

## Handoff

- `tests/installer_state_safety.sh` uses a local archive and mocked `curl`; it
  is deterministic and performs no network request.
- Move the large AGENTS block to the tracked generated template during CL-50;
  CL-13 preserves existing behavior until that cutover.

## Rollback

Remove only the managed `.gitignore` block or revert installer source. Do not
delete `.harness-id`, `harness.db`, backups, or target product files.
