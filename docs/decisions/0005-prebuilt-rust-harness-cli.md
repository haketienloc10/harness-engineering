# 0005 Prebuilt Rust Harness CLI

Date: 2026-05-23

## Status

Accepted, amended 2026-05-31, amended 2026-06-09, amended 2026-06-30

## Context

The durable layer started as a thin shell wrapper around SQLite. That wrapper is
now large enough to carry meaningful architecture risk: it mixes command
parsing, SQL construction, migrations, import behavior, query rendering, and
help text in one script.

The previous installer copied a shell wrapper into target repositories. That
kept Harness easy to install, but it also meant a Rust rewrite was not only an
implementation change. It changed the distribution contract for every project
that receives Harness.

## Decision

The Rust implementation of the Harness CLI is installed as a direct binary at
the repository-local command path.

The command path for users and agents is the installed Rust binary:

```bash
_harness/bin/harness-cli <command>
```

On Windows, the repository-local binary is installed as:

```powershell
.\_harness\bin\harness-cli.ps1 <command>
```

As of 2026-06-30, this source repository uses `install-harness-cli.sh` as the
CLI bootstrap path. The script builds `harness-cli` with Cargo in release mode
and copies `target/release/harness-cli` directly to `_harness/bin/harness-cli`.
There is no shell wrapper command contract and no `_harness/bin/harness-cli.bin`
runtime dependency.

The Rust CLI should follow the existing architecture rules:

- Domain: harness records, statuses, lanes, and value types.
- Application: use cases for intake, stories, decisions, backlog, traces, and
  queries.
- Infrastructure: SQLite repositories and schema migrations.
- Interface: command-line parsing, terminal output, and installer integration.

Release or install automation must preserve the same command contract: target
repositories use `_harness/bin/harness-cli` as the executable path.

## Alternatives Considered

1. Keep the shell CLI permanently. Rejected because the script has crossed from
   a thin wrapper into a growing application surface with weak testability.
2. Copy Rust source into every target project and build locally. Rejected
   because it makes Harness installation depend on a local Rust toolchain and
   increases setup friction for projects that only need the harness.
3. Require users to install a global `harness` binary separately. Rejected
   because Harness should remain repository-local for agents.
4. Download a prebuilt binary through the installer. Previously accepted, then
   superseded for this repository by the direct build bootstrap because the
   lightweight root installer no longer owns CLI download/build behavior.

## Consequences

Positive:

- The durable-layer CLI can move to typed command parsing and tested use cases.
- Target projects can execute the repository-local command without a wrapper
  indirection when the binary is present in the payload.
- The `_harness/bin/harness-cli` command is the stable entrypoint for
  agents on macOS/Linux; Windows uses the same repo-local path with the `.exe`
  suffix.
- The source repository can regenerate the exact local binary with one script.

Tradeoffs:

- `install-harness-cli.sh` requires a local Rust toolchain.
- A checked-in or copied binary is platform-specific.
- Cross-platform binary distribution remains a separate release concern.

## Follow-Up

- Treat the Rust CLI as the primary durable-layer implementation.
- Keep `_harness/bin/harness-cli` as the only local command path.
- Decide separately whether future release automation publishes additional
  platform binaries.
