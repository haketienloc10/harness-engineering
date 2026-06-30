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

The installer follows the repo-harness lightweight install mechanism: it
downloads the source archive from `haketienloc10/harness-engineering`, targets
`$PWD` by default or `HARNESS_LITE_TARGET_DIR` when set, installs the shared
scaffold whitelist, keeps existing target files outside `_harness/`, updates
`_harness/`, embeds or refreshes the Harness block in target `AGENTS.md`, filters
source-only artifacts, and writes `_harness/.harness-manifest`.

CLI build/install is handled by `install-harness-cli.sh`: it builds the Rust CLI
with Cargo and copies the release binary directly to `_harness/bin/harness-cli`.

## Relevant Product Docs

- `README.md`
- `install.sh`

## Acceptance Criteria

- A root `install.sh` supports the one-line curl install command.
- The installer source defaults to `haketienloc10/harness-engineering`.
- Existing target `AGENTS.md` is preserved and receives an idempotent Harness block.
- Existing target files outside `_harness/` are preserved.
- Installer records installed payload files in `_harness/.harness-manifest` and skips source-only artifacts.
- Installer output names the source, target, copied payload, skipped artifacts, kept files, and missing optional scaffold items.
- `install-harness-cli.sh` installs the local Rust CLI binary directly at `_harness/bin/harness-cli`.

## Design Notes

- Commands: `HARNESS_LITE_TARGET_DIR=/path/to/target install.sh`;
  `./install-harness-cli.sh`.
- Queries: none.
- API: none.
- Tables: none.
- Domain rules: update Harness runtime files under `_harness/`; add missing Harness-managed scaffold files without overwriting existing target files outside `_harness/`; do not touch unrelated app folders.
- UI surfaces: shell output only.

## Validation

| Layer       | Expected proof |
| ----------- | -------------- |
| Unit        | `bash -n install.sh`; `bash -n install-harness-cli.sh` |
| Integration | local install into a temporary target |
| E2E         | not applicable |
| Platform    | Linux shell execution in this workspace |
| Release     | not run; requires published GitHub raw script and release assets |

## Harness Delta

Replaced the previous option-heavy installer surface with a root `install.sh`
and removed the old Bash/PowerShell installer scripts from the Harness payload.

## Evidence

- `bash -n install.sh` passed.
- 2026-06-30 follow-up: installer behavior was aligned with
  `/home/locdt/Notes/VSCode/repo-harness/install.sh`, with the source path set
  to `haketienloc10/harness-engineering`.
- `HARNESS_LITE_TARGET_DIR="$tmp" ./install.sh` installed the GitHub archive
  payload into a temporary directory, wrote manifest source
  `haketienloc10/harness-engineering`, embedded the Harness markers in
  `AGENTS.md`, skipped source artifacts, and reported missing optional `.agents`.
- `./install-harness-cli.sh` built `harness-cli` with Cargo and installed the
  binary at `_harness/bin/harness-cli`.
- `_harness/bin/harness-cli --version` passed after direct binary install.
- `cargo test` passed for the Harness CLI crate.
- `git diff --check` passed.
