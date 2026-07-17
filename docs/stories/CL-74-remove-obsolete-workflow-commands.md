# CL-74 Remove Obsolete Workflow Commands

## Status

in_progress

## Lane and Scope

- Lane: normal
- Risk flags: existing behavior
- In scope: remove legacy commands from the agent-facing required workflow and
  align the runtime references with the command-first lifecycle.
- Out of scope: deleting compatibility, migration, audit, or diagnostic CLI
  commands without a separately approved compatibility decision.

## Product Contract

The required agent workflow is expressed through task lifecycle commands:
`task start`, context acknowledgement, `proof run`, and `task finish`.
Legacy standalone commands are not presented as required lifecycle steps.

## Acceptance Criteria

- Agent-facing runtime instructions name the command-first lifecycle.
- No required-loop example directs an agent to use standalone `intake` or
  `trace` as lifecycle roots or completion transitions.
- Legacy commands remain available only where their supported purpose is
  explicitly documented.

## Design and Decisions

This is a documentation and workflow-surface correction. CLI compatibility is
preserved because existing operational records and migration/audit workflows may
still depend on those commands.

## Human Gates

- none.

## Validation and Evidence

| Layer | Expected proof | Result |
| --- | --- | --- |
| Unit | Workflow policy and command help | not run |
| Integration | Search for legacy required-loop commands | not run |

## Rollback and Harness Delta

Revert the documentation-only changes to restore the former presentation.
The harness delta is that agent instructions follow the command-first lifecycle
without implying removal of supported compatibility commands.
