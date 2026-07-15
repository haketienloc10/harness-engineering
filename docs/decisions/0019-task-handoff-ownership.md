# 0019 Task Handoff Ownership

Date: 2026-07-14

## Status

Accepted

## Decision

An owned active task may transition its owner only through `task handoff`.
The command requires the current owner, target owner, source and evidence; it
updates the owner and inserts a `task_approval` row with gate `handoff` in one
SQLite transaction. Subsequent lifecycle transitions require a matching
`--owner` whenever a task has an owner.

## Consequences

The CLI records a claimed handoff but does not cryptographically verify human
identity. Ownerless legacy tasks remain operable; session/lease expiry is a
later contract and must not be inferred from owner text.
