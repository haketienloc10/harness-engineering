# 0008 Simple Curl Installer

Date: 2026-06-29

## Status

Accepted

## Context

The previous installer exposed many routine choices: target flags, merge and
override modes, shim refresh options, PowerShell parity, dry-run behavior, and
release download controls. That made the install path harder to explain than
the Harness payload itself.

The desired install experience is a single curl-piped command:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/install.sh?$(date +%s)" | bash
```

## Decision

Use a root `install.sh` as the primary install contract. By default it installs
into the current directory. It may accept one optional positional target
directory for local testing or scripted setup.

The installer should:

- Download the repository snapshot when run through curl.
- Use the local checkout as source when run as `./install.sh`.
- Back up existing `AGENTS.md` and `_harness/` before replacing them.
- Append Harness local-state rules to `.gitignore`.
- Try to install the platform CLI binary, but remove it and report a clear
  follow-up if the binary cannot run on the target system.
- Build the CLI from `crates/harness-cli` as a fallback when release binary
  installation fails and `cargo` plus source files are available.

Do not reintroduce the old option-heavy installer surface unless a concrete
target repository need requires it.

## Alternatives Considered

1. Keep the previous option-heavy installer script with many flags. Rejected
   because routine install choices should not be pushed to the human.
2. Require users to clone this repo before installing. Rejected because the
   install contract should work from one shell command.
3. Keep Windows PowerShell installer parity now. Deferred because the requested
   contract is curl-to-bash and the Harness payload should be simplified first.

## Consequences

Positive:

- The install command is short enough to paste into a target repo.
- Existing Harness surfaces are backed up automatically.
- The payload no longer carries multiple installer implementations.

Tradeoffs:

- Windows-specific installation is no longer a first-class script in the
  payload.
- Advanced merge and dry-run workflows are removed from the primary path.
- CLI install can still succeed without release assets when Rust source and
  `cargo` are available.
- Source-build fallback takes longer than copying a release binary.

## Follow-Up

- Ensure `crates/harness-cli` source files are tracked in this repository if
  source-build fallback is expected to work from the curl installer.
