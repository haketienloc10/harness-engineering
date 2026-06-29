# US-001 Simple Curl Installer

## Status

implemented

## Lane

normal

## Product Contract

Harness installs from one curl-piped root script:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/install.sh?$(date +%s)" | bash
```

The installer targets the current directory by default, accepts one optional
target directory argument, backs up existing `AGENTS.md`, legacy
`.agent-harness/`, and `_harness/`, then replaces `_harness/` with the current
Harness runtime. It snapshots `docs/` and updates it file by file without
deleting target-only files, appends local `.gitignore` rules, and installs the
CLI by downloading a compatible release binary or building from
`crates/harness-cli` when source and `cargo` are available.

## Relevant Product Docs

- `README.md`
- `install.sh`

## Acceptance Criteria

- A root `install.sh` supports the one-line curl install command.
- The documented install path no longer requires the removed option-heavy installer.
- Existing target `AGENTS.md`, legacy `.agent-harness/`, and `_harness/` are backed up before replacement.
- Existing `docs/` is snapshotted before file-level updates, and target-only docs are preserved.
- Installer output names the source, target, installed payload, and next CLI commands.

## Design Notes

- Commands: `install.sh [target-dir]`.
- Queries: none.
- API: none.
- Tables: none.
- Domain rules: replace Harness runtime under `_harness/`; update Harness-managed product record files under `docs/` without deleting target-only docs; do not touch unrelated app folders.
- UI surfaces: shell output only.

## Validation

| Layer       | Expected proof |
| ----------- | -------------- |
| Unit        | `bash -n install.sh` |
| Integration | local install into a temporary target |
| E2E         | not applicable |
| Platform    | Linux shell execution in this workspace |
| Release     | not run; requires published GitHub raw script and release assets |

## Harness Delta

Replaced the previous option-heavy installer surface with a root `install.sh`
and removed the old Bash/PowerShell installer scripts from the Harness payload.

## Evidence

- `bash -n install.sh` passed.
- `cargo test` passed for the Harness CLI crate.
- `./install.sh "$tmp"` installed `AGENTS.md`, `_harness/`, `docs/product/`,
  `docs/stories/`, `docs/decisions/`, schema files, CLI wrapper, and
  `.gitignore` rules into a temporary target.
- `(cd "$tmp" && _harness/bin/harness-cli init &&
  _harness/bin/harness-cli query matrix)` passed after install.
- `./install.sh "$tmp"` with pre-existing `AGENTS.md`, legacy
  `.agent-harness/`, `_harness/`, and managed product record directories backed
  those paths up under `.harness-backup/<timestamp>/` before replacing them.
- Temporary install confirmed old option-heavy installer files were not present
  in the installed payload.
- `git diff --check` passed.
- CLI binary download fallback reached a release asset, but the binary could
  not run in this environment because of the system libc version; installer
  removed the unusable binary and tried the source-build fallback.
- Source-build fallback expects either root workspace `Cargo.toml` with package
  `harness-cli` or `crates/harness-cli/Cargo.toml`; this checkout has tracked
  Rust source under `crates/harness-cli/`, and the fallback installed the
  `_harness/bin/harness-cli` command successfully in the temporary target.
- 2026-06-29 follow-up: installer behavior was changed to replace `_harness/`
  as a full runtime tree while updating `docs/` file by file and preserving
  target-only documentation.
