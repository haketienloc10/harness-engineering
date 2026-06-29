# Agent Harness Map

This directory is agent-owned. Read and update it as operational memory for the
target repository.

## Policy Files

- `HARNESS.md`: operating index, core loop, and durable-layer contract.
- `FEATURE_INTAKE.md`: input types, risk flags, and lane selection.
- `CONTEXT_RULES.md`: read targets by phase and lane.
- `ARCHITECTURE.md`: boundary and layering rules.
- `TOOL_REGISTRY.md`: optional tool capability lookup and degrade rules.
- `TRACE_SPEC.md`: trace fields and required detail by lane.
- `TEST_MATRIX.md`: fallback proof map; prefer
  `_harness/bin/harness-cli query matrix`.
- `HARNESS_AUDIT.md`: durable-state drift checks and entropy score.
- `HARNESS_COMPONENTS.md`: workflow component taxonomy and file inventory.
- `HARNESS_MATURITY.md`: measurable H0-H5 harness capability ladder.
- `IMPROVEMENT_PROTOCOL.md`: friction-to-proposal improvement loop.

## Product Record Directories

- `docs/product/`: living product contract derived from accepted input.
- `docs/stories/`: story packets and progress evidence.
- `docs/decisions/`: durable decision records.

## Harness Support Directories

- `templates/`: required formats for specs, stories, decisions, high-risk work,
  and validation reports.

## Rule

Do not put application documentation here unless it is part of agent-operable
product truth. Do not put marketing docs, onboarding docs, or general human
notes here.
