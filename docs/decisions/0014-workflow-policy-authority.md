# 0014 Workflow Policy Authority

Date: 2026-07-14

## Status

Accepted

## Decision

`_harness/workflow.toml` becomes the single machine-readable source for
deterministic lane, gate, materiality, and context policy. ADRs explain policy
rationale; CLI validates and renders it. Rust and generated installed content
must not duplicate lane thresholds or policy literals.

## Consequences

The command-first path ships in parity/shadow mode before old Markdown policy
is compacted or removed.
