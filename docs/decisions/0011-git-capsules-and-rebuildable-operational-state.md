# 0011 Git Capsules and Rebuildable Operational State

Date: 2026-07-14

## Status

Accepted

## Decision

Git-tracked product docs, stories, decisions, and task capsules are semantic
truth. `harness.db` is an ignored local operational index/event store and must
be rebuildable from those artifacts for critical project memory. A terminal
material task requires a validated task capsule; raw SQLite is never versioned.

## Consequences

Rebuild is explicit, conflict-safe, and never silently overwrites a DB. DB-only
records are operational evidence, not canonical product truth.
