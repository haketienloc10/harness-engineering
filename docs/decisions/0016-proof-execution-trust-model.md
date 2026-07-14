# 0016 Proof Execution Trust Model

Date: 2026-07-14

## Status

Accepted

## Decision

Proof runs store an executable and argv as structured data and execute from the
canonical repository root or a declared safe working directory. Shell execution
requires an explicit opt-in and warning. A passing proof is current only when
its exact commit and dirty-worktree fingerprint match the task finish state.

## Consequences

Proof records are append-only. A failed run cannot be overwritten by a boolean,
and `not_applicable` requires a declared reason or validation plan.
