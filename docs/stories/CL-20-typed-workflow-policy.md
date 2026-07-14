# CL-20 Typed Workflow Policy

Status: completed

## Implemented So Far

- `workflow.toml` now parses through strict serde/TOML types with unknown keys
  rejected.
- Policy major version, repo-relative paths, thresholds, lane tiers, story and
  capsule enums, and non-empty proof lists are validated.
- `workflow validate --json` and `workflow explain --json` render stable
  machine-readable output.

## Validation Evidence

2026-07-14: `cargo test -p harness-cli` (48 passed), clippy, fmt, and live
`workflow validate|explain --json` passed.

The corpus covers unknown keys, path traversal, tiny/normal thresholds, and a
hard-gate high-risk classification.

## Handoff

CL-21 owns context acknowledgement/materiality fields and use of the stored
policy in task context manifests.
