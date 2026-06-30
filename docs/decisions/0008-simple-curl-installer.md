# 0008 Simple Curl Installer

Date: 2026-06-29

## Status

Accepted, amended 2026-06-29, amended 2026-06-30

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
into the current directory. It may accept `HARNESS_LITE_TARGET_DIR` for local
testing or scripted setup.

As of 2026-06-30, align the root installer with the lightweight
`repo-harness/install.sh` mechanism while keeping this repository as the source:

- Download the repository snapshot from `haketienloc10/harness-engineering`
  through `codeload.github.com`.
- Target the current directory by default, or `HARNESS_LITE_TARGET_DIR` when set.
- Install the shared scaffold whitelist.
- Update `_harness/` files, but preserve existing target files outside
  `_harness/`.
- Embed or refresh a marked Harness block inside target `AGENTS.md` instead of
  copying the source `AGENTS.md`.
- Filter source-only artifacts from `docs/`, `_harness/`, generated knowledge
  docs, and durable local state.
- Write `_harness/.harness-manifest` with the installed payload file list.
- Leave CLI build/install behavior to `install-harness-cli.sh`, which builds the
  Rust CLI from source and copies the binary directly to `_harness/bin/harness-cli`.

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
- Target-specific `AGENTS.md` content is preserved while Harness instructions
  are refreshed idempotently.
- Source-specific story, decision, and generated docs are filtered from target
  installs.

Tradeoffs:

- Windows-specific installation is no longer a first-class script in the
  payload.
- Advanced merge and dry-run workflows are removed from the primary path.
- This installer no longer performs CLI binary download or inline source-build
  fallback; CLI bootstrapping is handled by `install-harness-cli.sh`.
- Target directory selection uses an environment variable rather than a
  positional argument.

## Follow-Up

- Future release automation may publish platform binaries, but the local source
  bootstrap is `./install-harness-cli.sh`.
