---
schema: harness/story/v1
id: CL-01
title: Canonical Architecture Decision Package
status: implemented
lane: high_risk
product_docs:
  - docs/decisions/0010-main-schema-lineage-without-symphony.md
  - docs/decisions/0011-git-capsules-and-rebuildable-operational-state.md
  - docs/decisions/0012-task-lifecycle-and-closure-invariants.md
  - docs/decisions/0013-concern-specific-source-hierarchy.md
  - docs/decisions/0014-workflow-policy-authority.md
  - docs/decisions/0015-trace-and-capsule-privacy.md
  - docs/decisions/0016-proof-execution-trust-model.md
  - docs/decisions/0017-explicit-code-impact-classification.md
---
# CL-01 Canonical Architecture Decision Package

## Status

implemented

## Lane

high-risk

## Product Contract

The accepted main-lineage, portable-memory, lifecycle, source-hierarchy,
workflow-authority, privacy, proof-trust and code-impact decisions form the
canonical architecture package for CLP-001.

This artifact records the current canonical contract and current validation.
It does not claim that a separate historical CL-01 lifecycle task existed.

## Acceptance Criteria

- Decisions 0010 through 0017 remain present and accepted.
- Current artifact validation discovers this story and all referenced decision
  files without duplicate identity, unsafe path or missing-reference errors.
- Current CLP-001-R1 reconciliation proof validates the package without
  manufacturing historical task, trace, proof or approval records.

## Evidence

The current reconciliation is owned by `TASK-000027` under CL-72. Its named
coverage proofs, detailed trace and v2 capsule are current evidence only.

## Rollback

Remove this current reconciliation artifact and its supported story projection
as one change. Preserve all historical durable records and accepted decisions.
