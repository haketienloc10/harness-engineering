# US-002 Harness Runtime and Docs Paths

## Status

implemented

## Lane

normal

## Product Contract

Agents operate the harness runtime from `_harness/`. Product records owned by
the target repository live under `docs/product/`, `docs/stories/`, and
`docs/decisions/`.

## Relevant Product Docs

- `README.md`
- `AGENTS.md`
- `docs/decisions/0009-harness-runtime-and-product-record-paths.md`

## Acceptance Criteria

- The installed harness runtime directory is `_harness/`, not `.agent-harness/`.
- Product docs, stories, and decisions are installed under `docs/`.
- Agent instructions, templates, source hierarchy, and trace examples reference
  the new paths.
- The CLI loads schema migrations from `_harness/scripts/schema/`.
- CLI help renders usage as `_harness/bin/harness-cli <COMMAND>`, not the
  internal executable filename.
- The installer backs up legacy `.agent-harness/` and installs the new docs
  directories.

## Design Notes

- Commands: `_harness/bin/harness-cli <command>`.
- Queries: `_harness/bin/harness-cli query matrix`.
- API: none.
- Tables: no schema shape change.
- Domain rules: `_harness/` is agent runtime; `docs/` is product record.
- CLI internals may use an ignored local executable, but user-facing docs,
  help, and installer output expose only `_harness/bin/harness-cli`.
- UI surfaces: shell output only.

## Validation

| Layer       | Expected proof |
| ----------- | -------------- |
| Unit        | `cargo test` |
| Integration | local install into a temporary target |
| E2E         | not applicable |
| Platform    | `bash -n install.sh`; installed CLI `init` and `query matrix` |
| Release     | not run; requires published GitHub raw script and release assets |

## Harness Delta

Renamed the harness runtime path to `_harness/`, moved product record
directories to `docs/`, updated installer behavior, and recorded decision 0009.

## Evidence

- `cargo test` passed.
- `bash -n install.sh` passed.
- `git diff --check` passed.
- Local install into a temporary target installed `AGENTS.md`, `_harness/`,
  `docs/product/`, `docs/stories/`, and `docs/decisions/`; it did not install
  `.agent-harness/`.
- Local upgrade smoke with pre-existing `.agent-harness/`, `_harness/`, and
  managed product record directories backed up the legacy and replaced paths.
- Installed target `(_harness/bin/harness-cli init &&
  _harness/bin/harness-cli query matrix)` passed.
- 2026-06-29 follow-up: `cargo test -p harness-cli`, `cargo build --release
  -p harness-cli`, `bash -n install.sh`, `git diff --check`, direct
  `_harness/bin/harness-cli help`, subcommand help, and local install smoke
  passed with usage/output exposing only `_harness/bin/harness-cli`.
