# CL-75 Remove Legacy CLI Commands

## Status

completed

## Lane and Scope

- Lane: high-risk
- Risk flags: public contracts, existing behavior
- In scope: remove the compatibility-window CLI commands identified by the
  command-first execution plan: `init`, `migrate`, `intake`, `import brownfield`,
  standalone `trace`, `score-trace`, `score-context`, and `story`
  `add`/`update`/`verify`/`verify-all` subcommands. Update all callers, tests,
  packaged payloads, and agent-facing documentation.
- Out of scope: removing active command-first lifecycle commands, read-only
  artifact checks, or unrelated operational commands.

## Product Contract

The public CLI exposes the command-first lifecycle and its active support
commands only. It has no compatibility window for the listed legacy commands.

## Acceptance Criteria

- `workflow commands` and CLI help omit every listed legacy command.
- No shipped documentation or test invokes a removed command.
- The rework workflow remains executable through `task`, `proof`, `memory`,
  `friction`, `audit`, and the active `story check` surface.
- Source and packaged CLI surfaces agree.

## Design and Decisions

The removed commands had no lifecycle ownership after rework. `task start`
owns intake creation; `task finish` owns terminal trace selection and closure;
proof is recorded through `proof run`. `story check` remains as artifact
validation, while legacy mutable story commands are removed.

## Human Gates

- Approved by user on 2026-07-17: remove all commands no longer used by the
  reworked workflow.

## Validation and Evidence

| Layer | Expected proof | Result |
| --- | --- | --- |
| Unit | Rust CLI tests | pass: 79 tests |
| Integration | CLI command tree and workflow parity | pass |
| Release | Packaged CLI and installer qualification | pass: packaged command tree and parity |

## Rollback and Harness Delta

Restore the removed command variants and their callers as one compatibility
surface. This intentionally breaks callers of the retired commands.

## Follow-up

CL-77 amends ADR 0025 by restoring only `init` as a record-free database
bootstrap boundary. The remaining legacy commands stay removed, so this
completed story's command-first lifecycle outcome remains intact.
