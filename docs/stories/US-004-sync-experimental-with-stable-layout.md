# US-004 Sync Experimental With Stable Layout

## Status

in_progress

## Lane

normal

## Product Contract

`harness-engineering` carries the current CLI, schema, installer, and Symphony
behavior from `harness-experimental` without changing its `_harness/` runtime
layout or repository identity.

## Relevant Product Docs

- `README.md`
- `_harness/HARNESS.md`
- `_harness/ARCHITECTURE.md`
- `docs/decisions/0010-experimental-sync-with-stable-layout.md`

## Acceptance Criteria

- Current experimental CLI and schema behavior are available under existing
  `_harness/` paths.
- Symphony uses `_harness/bin/harness-cli` and `_harness/scripts/schema/`.
- Installer and agent instructions preserve the current public path contract.
- Experimental build output and unrelated untracked files are excluded.
- Both repositories record the paired synchronization baseline.

## Design Notes

- Commands: extend `harness-cli`; add `harness-symphony`.
- Queries: preserve existing commands and add experimental query behavior.
- Tables: apply schema migrations `006` through `008`.
- Domain rules: translate source paths; do not translate product record paths.
- UI surfaces: Symphony web controller.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | `cargo test --workspace` |
| Integration | CLI init/audit/rebuild and changeset tests |
| E2E | Symphony web UI tests where environment permits |
| Platform | installer syntax and dry-run smoke |
| Release | release workflow and asset naming inspection |

## Harness Delta

Adds a repeatable cross-repository synchronization baseline while retaining the
accepted runtime layout.

## Evidence

- Source baseline:
  `harness-experimental@14e6f102a4a645562d046f7c693c61401261cac6`.
- `CARGO_HOME=/tmp/harness-engineering-cargo cargo test --workspace`: 134
  tests passed.
- `CARGO_HOME=/tmp/harness-engineering-cargo cargo clippy --workspace
  --all-targets -- -D warnings`: passed.
- `npm run build`: passed for Symphony web UI.
- `CARGO_HOME=/tmp/harness-engineering-cargo
  PLAYWRIGHT_BROWSERS_PATH=/tmp/harness-playwright npm run e2e`: 19 tests
  passed.
- `bash -n install.sh`; `bash -n install-harness-cli.sh`; `git diff --check`:
  passed.
- `_harness/bin/harness-cli migrate`: applied migrations 006 through 008.
- Experimental UI test fixture was made repository-independent instead of
  relying on source-local `harness.db` story `US-052`.
- Commit creation remains pending because this execution environment exposes
  both repositories' `.git` directories as read-only. The destination source
  tree is complete and validated; the paired source marker must be created
  after the destination commit hash exists.
