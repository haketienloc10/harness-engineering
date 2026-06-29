# Harness Maturity Ladder

This ladder tracks how `harness-engineering` moves from static instructions to
measurable self-improvement. A level is achieved only when repository files,
durable records, or validation output prove it.

## Levels

| Level | Name | Criteria | Current Status |
| --- | --- | --- | --- |
| H0 | Bare environment | No `AGENTS.md`, intake policy, story, decision, validation, or trace artifact. | Passed. This repo is beyond H0. |
| H1 | Scaffolding and policy | `AGENTS.md`; `_harness/HARNESS.md`; `_harness/FEATURE_INTAKE.md`; `_harness/ARCHITECTURE.md`; templates; fallback matrix. | Achieved. |
| H2 | Durable state and observability | `_harness/bin/harness-cli`; schema for intake/story/decision/backlog/trace; `_harness/TRACE_SPEC.md`; `_harness/CONTEXT_RULES.md`. | Achieved. |
| H3 | Active observability and evolution | `score-trace`; friction queries; backlog predicted/actual outcomes; `audit`; `propose`; component attribution. | Partial: benchmark attribution is not automated. |
| H4 | Automated verification | Story `verify_command`; `story verify`; `story verify-all`; trace warnings for unverified linked stories. | Achieved. |
| H5 | Self-improving harness | Repeated friction becomes proposed change with predicted impact, risk, validation plan, review posture, and actual outcome comparison. | Partial: repeated outcome proof is still sparse. |

## Activated Responsibilities

| Level | Responsibilities |
| --- | --- |
| H1 | Task specification, permissions, project memory, verification |
| H2 | Task state, observability, failure attribution, context selection, entropy auditing |
| H3 | Observability, failure attribution, entropy auditing, intervention recording |
| H4 | Verification, task state, permissions, intervention recording |
| H5 | Entropy auditing, failure attribution, intervention recording, permissions |

## Current Assessment

| Level | Status | Evidence |
| --- | --- | --- |
| H0 | Passed | Harness docs, templates, and durable records exist. |
| H1 | Achieved | `AGENTS.md`, `_harness/HARNESS.md`, `_harness/FEATURE_INTAKE.md`, `_harness/ARCHITECTURE.md`, `_harness/templates/*`, `_harness/TEST_MATRIX.md`. |
| H2 | Achieved | `_harness/bin/harness-cli`, `_harness/scripts/schema/001-init.sql`, `_harness/TRACE_SPEC.md`, `_harness/CONTEXT_RULES.md`. |
| H3 | Partial | `score-trace`, `query friction`, backlog outcome fields, `audit`, and `propose` exist; benchmark attribution remains open. |
| H4 | Achieved | `story verify`, `story verify-all`, and trace-time verification warnings exist. |
| H5 | Partial | `audit`, `score-context`, `intervention`, `propose`, and `_harness/IMPROVEMENT_PROTOCOL.md` exist; repeated outcome proof remains open. |
