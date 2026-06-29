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
target directory argument, backs up existing `AGENTS.md` and `.agent-harness/`,
installs the Harness payload, appends local `.gitignore` rules, and installs the
CLI by downloading a compatible release binary or building from
`crates/harness-cli` when source and `cargo` are available.

## Relevant Product Docs

- `README.md`
- `.agent-harness/install/README.md`

## Acceptance Criteria

- A root `install.sh` supports the one-line curl install command.
- The documented install path no longer requires `.agent-harness/install/install-harness.sh --directory ... --yes`.
- Existing target `AGENTS.md` and `.agent-harness/` are backed up before replacement.
- Installer output names the source, target, installed payload, and next CLI commands.

## Design Notes

- Commands: `install.sh [target-dir]`.
- Queries: none.
- API: none.
- Tables: none.
- Domain rules: install only Harness runtime surface, not product `docs/` or app folders.
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
- `./install.sh "$tmp"` installed `AGENTS.md`, `.agent-harness/`, schema files,
  and `.gitignore` rules into a temporary target.
- `./install.sh "$tmp"` with pre-existing `AGENTS.md` and `.agent-harness/`
  backed both paths up under `.harness-backup/<timestamp>/` before replacing
  them.
- Temporary install confirmed old `.agent-harness/install/install-harness.sh`
  was not present in the installed payload.
- `git diff --check` passed.
- CLI binary download fallback reached a release asset, but the binary could
  not run in this environment because of the system libc version; installer
  removed the unusable binary and tried the source-build fallback.
- Source-build fallback could not run in this checkout because
  `crates/harness-cli/Cargo.toml` is not present in the working tree.
