# CL-20 Typed Workflow Policy

Status: completed

## Implemented So Far

- `workflow.toml` now parses through strict serde/TOML types with unknown keys
  rejected.
- Policy major version, repo-relative paths, thresholds, lane tiers, story and
  capsule enums, and non-empty proof lists are validated.
- `workflow validate --json` and `workflow explain --json` render stable
  machine-readable output.
- Policy mode is explicit (`shadow|authority`), flag aliases normalize current
  Markdown vocabulary, and typed context rules cover phases, lanes, triggers,
  must/should/skip outputs, stop condition and token budgets.

## Validation Evidence

2026-07-14: `cargo test -p harness-cli` (53 passed), clippy, fmt, and live
`workflow validate|explain --json` passed.

The corpus covers unknown keys, path traversal, tiny/normal thresholds, and a
hard-gate high-risk classification.

## Handoff

CL-21 owns the pure context manifest compiler. CL-40 owns task persistence and
CL-41 owns acknowledgement/refresh, as recorded in the CLP-001 amendment.
