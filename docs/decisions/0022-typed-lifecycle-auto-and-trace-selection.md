# 0022 Typed Lifecycle Auto Classification and Trace Selection

Date: 2026-07-15

## Status

Accepted

## Context

The documented minimal `task start` and `task finish` paths diverged from the
CLI because behavior classification and trace selection were mandatory caller
arguments. Free-text summary inference would make lifecycle state
non-deterministic and unauditable.

## Decision

- `--behavior-bearing` accepts `auto|yes|no` and defaults to `auto`.
- `auto` uses only normalized typed intake, explicit flags, and a linked story.
  It records reasons and never reads summary prose.
- Explicit non-behavior flags may classify maintenance/docs/read-only work as
  false. A linked story is conservatively behavior-bearing.
- When owner and session are both omitted, the CLI creates an explicit
  `harness-cli` owner with a unique returned session. Supplying only one is a
  usage error.
- Missing required stories become visible unmet gates with exact remediation;
  start remains atomic and finish remains fail-closed.
- `task finish` accepts an explicit `--trace` or, when omitted, selects exactly
  one qualifying trace rooted in the task intake. Zero or multiple qualifying
  traces fail with structured remediation. Trace content is never synthesized.

The user explicitly approved all CLP-001-R1 human gates on 2026-07-15;
`TASK-000025` stores the approval source, evidence, and scope.

## Alternatives Considered

1. Infer from summary language: rejected as unstable semantic guessing.
2. Keep mandatory `yes|no` and `--trace`: rejected because documented minimal
   commands remain non-executable.
3. Pick the newest trace when several qualify: rejected because it hides
   ambiguity and weakens recovery determinism.

## Consequences

Start/status expose classification, ownership, gates, context and remediation.
Legacy explicit arguments remain valid through the N+2 compatibility window.

## Follow-Up

- Prove source, packaged and installed CLI parity.
- Preserve explicit `--trace` for deterministic recovery and idempotency.

