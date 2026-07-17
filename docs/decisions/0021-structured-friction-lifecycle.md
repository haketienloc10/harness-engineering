# 0021 Structured Friction Lifecycle

Date: 2026-07-14

## Status

Accepted

## Decision

Material harness friction is an operational record, not free-text trace
metadata. Each record has a stable content fingerprint, category, severity,
disposition and lifecycle state: `proposed`, `accepted`, `in_progress`,
`implemented_pending_observation`, `validated`, `ineffective`, or `reverted`.

Trace friction remains compatibility evidence only. Completion rejects an
unresolved material friction record linked to the task.

## Consequences

Migration `009-structured-friction.sql` adds the record without changing old
traces. Command-first add/resolve/query paths own mutations; later maturity and
release proof query these records.
