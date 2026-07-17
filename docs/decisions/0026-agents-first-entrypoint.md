# 0026 Agents-First Entrypoint

Date: 2026-07-17

## Status

Accepted

## Context

`AGENTS.md` is the first instruction surface loaded by agents, but it had
grown into explanatory prose for human readers. That obscures precedence,
commands, and stop conditions during execution.

## Decision

Keep `AGENTS.md` as a compact, imperative agents-first contract: source
precedence, mandatory lifecycle commands, vague-continuation behavior, and
hard stops only. Put rationale, examples, and detailed process descriptions in
the owned `_harness/` references.

## Alternatives Considered

1. Keep explanations inline in `AGENTS.md`: rejected because it competes with
   executable instructions for limited agent context.
2. Remove `AGENTS.md`: rejected because it is the portable entrypoint and
   installed contract.

## Consequences

Positive:

- Agents receive a shorter, deterministic execution contract first.
- Detail remains available from named, concern-owned references.

Tradeoffs:

- Human readers must open `_harness/HARNESS.md` or the cited references for
  rationale and examples.

## Follow-Up

- Keep future `AGENTS.md` additions imperative and link detailed guidance
  instead of embedding it.
