# Harness Components

This taxonomy maps the current `harness-engineering` payload to the workflow
responsibilities agents rely on. It is adapted from the experimental harness
model, with runtime policy kept under `_harness/` and product records kept under
`docs/product/`, `docs/stories/`, and `docs/decisions/`.

Status values:

- **Covered**: the repository has an explicit file, command, or record for this
  responsibility.
- **Partial**: support exists, but is advisory, manual, or not yet measured.
- **Missing**: no meaningful support exists yet.

## Responsibility Map

| # | Responsibility | Status | Harness Files | Evidence | Gap |
| --- | --- | --- | --- | --- | --- |
| 1 | Task specification | Covered | `AGENTS.md`, `_harness/FEATURE_INTAKE.md`, `_harness/templates/*`, `docs/stories/*`, `intake` and `story` tables | Requests are classified by type and lane before implementation; normal and high-risk work have templates and durable story rows. | Keep story packets synchronized with future product docs. |
| 2 | Context selection | Covered | `AGENTS.md`, `_harness/CONTEXT_RULES.md`, `_harness/ARCHITECTURE.md`, `docs/decisions/*`, `_harness/bin/harness-cli score-context` | Phase-by-lane context rules and retrieval triggers are documented and measurable. | Future automation could enforce context selection instead of only measuring it. |
| 3 | Tool access | Covered | `_harness/bin/harness-cli`, `_harness/TOOL_REGISTRY.md`, `tool` table, `crates/harness-cli/*`, `install.sh` | The CLI exposes compiled commands and a kind-aware inbound tool registry with presence scanning. | Permission profiles and usage analytics remain future work. |
| 4 | Project memory | Covered | `_harness/HARNESS.md`, `docs/decisions/*`, `docs/stories/*`, `harness.db`, `decision`, `backlog`, and `trace` tables | Decisions, backlog, stories, and traces preserve durable knowledge across tasks. | Old traces still require manual summarization. |
| 5 | Task state | Covered | `_harness/bin/harness-cli query matrix`, `_harness/TEST_MATRIX.md`, `intake`, `story`, and `trace` tables | Durable records track intake, story status, proof columns, and task traces. | Fallback markdown can drift and should be kept small. |
| 6 | Observability | Partial | `_harness/TRACE_SPEC.md`, `trace` table, `_harness/bin/harness-cli trace`, `score-trace`, `query traces`, `query friction` | Traces are scored when recorded and can be queried with friction context. | No dashboard or benchmark ingestion exists in this payload. |
| 7 | Failure attribution | Partial | `_harness/HARNESS_COMPONENTS.md`, `_harness/TRACE_SPEC.md`, `trace.errors`, `trace.harness_friction`, `backlog` table | Failures can be tied to files, components, friction, backlog proposals, and linked intake context. | No automated attribution from benchmark failures to harness components exists yet. |
| 8 | Verification | Covered | `_harness/TEST_MATRIX.md`, `query matrix`, `story verify`, `story verify-all`, `trace`, `score-trace`, `story.verify_command`, `_harness/templates/validation-report.md` | Stories can store and run mechanical proof commands individually or in batch, and traces warn when linked story verification has not passed. | Proof-column automation remains future work. |
| 9 | Permissions | Partial | `AGENTS.md`, `_harness/HARNESS.md`, `_harness/FEATURE_INTAKE.md`, `_harness/ARCHITECTURE.md`, installer backup rules | Policy describes when agents may update docs and when to ask before architecture or workflow changes. | Permissions are instruction-level only; no enforced command allowlist exists. |
| 10 | Entropy auditing | Covered | `_harness/HARNESS_AUDIT.md`, `_harness/IMPROVEMENT_PROTOCOL.md`, `backlog` table, `trace.harness_friction`, `audit`, `propose` | Audit detects drift, backlog items compare predicted impact to actual outcome, and proposals can become backlog items. | Automated repair remains future work. |
| 11 | Intervention recording | Covered | `intervention` table, `intervention add`, `query interventions`, `_harness/HARNESS.md` | Human, reviewer, CI, and agent interventions are separate durable records and can be filtered by trace, story, or type. | Capture is still manual and advisory. |

## File Inventory

| File | Primary Responsibility | Secondary Responsibilities |
| --- | --- | --- |
| `AGENTS.md` | Context selection | Task specification, permissions |
| `README.md` | Task specification | Project memory |
| `install.sh` | Tool access | Permissions, verification |
| `Cargo.toml`, `Cargo.lock` | Tool access | Verification |
| `crates/harness-cli/*` | Tool implementation | Task state, observability, verification |
| `_harness/ARCHITECTURE.md` | Permissions | Context selection, task specification |
| `_harness/FEATURE_INTAKE.md` | Task specification | Permissions, context selection |
| `_harness/HARNESS.md` | Task specification | Project memory, task state, permissions |
| `_harness/CONTEXT_RULES.md` | Context selection | Permissions, task specification |
| `_harness/TRACE_SPEC.md` | Observability | Failure attribution, intervention recording |
| `_harness/TOOL_REGISTRY.md` | Tool access | Context selection, verification |
| `_harness/TEST_MATRIX.md` | Verification | Task state |
| `_harness/HARNESS_AUDIT.md` | Entropy auditing | Verification, task state |
| `_harness/HARNESS_COMPONENTS.md` | Failure attribution | Observability, entropy auditing |
| `_harness/HARNESS_MATURITY.md` | Entropy auditing | Observability, verification |
| `_harness/IMPROVEMENT_PROTOCOL.md` | Entropy auditing | Failure attribution, permissions |
| `_harness/templates/*` | Task specification | Verification, project memory |
| `_harness/scripts/schema/*` | Task state | Observability, project memory |
| `docs/product/*` | Product contract | Project memory |
| `docs/stories/*` | Task specification | Verification, project memory |
| `docs/decisions/*` | Project memory | Permissions |

## Coverage Summary

- Covered: 8/11 responsibilities.
- Partial: 3/11 responsibilities.
- Missing: 0/11 responsibilities.

Partial responsibilities are observability, failure attribution, and
permissions because this payload records and scores operational data but does
not enforce policy or ingest benchmark failures automatically.
