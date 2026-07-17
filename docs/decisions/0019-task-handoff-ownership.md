# 0019 Task Handoff Ownership

Date: 2026-07-14

## Status

Accepted, amended 2026-07-15

## Decision

An owned active task may transition its owner or session only through
`task handoff`. The command requires distinct current/target owners and
sessions, source and evidence; it updates both identity fields, issues a fresh
bounded lease and inserts a `task_approval` row with gate `handoff` in one
SQLite transaction. Subsequent lifecycle transitions require matching
`--owner` and `--session` whenever a task has a session lease.

An active primary-story root remains exclusive after lease expiry. Expiry or
`task block` releases only the worktree execution claim; a different agent must
use handoff or wait for terminal disposition rather than create an unrecorded
parallel takeover.

## Consequences

The CLI records a claimed handoff but does not cryptographically verify human
identity. Ownerless and pre-migration owner-only tasks remain operable. New
session identity is explicit and must not be inferred from owner text.
