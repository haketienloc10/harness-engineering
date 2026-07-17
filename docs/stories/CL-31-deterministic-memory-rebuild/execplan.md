# CL-31 Exec Plan: Deterministic Memory Rebuild

## Goal

Rebuild operational projections from canonical Git artifacts into a new,
validated temporary SQLite database, with no implicit replacement of the
current database.

## Scope

In scope: `memory rebuild --dry-run`, conflict report, temporary database
creation, canonical story/decision/capsule projection, doctor/audit validation,
repeatable logical equivalence and explicit switch preparation.

Out of scope: automatic conversion of legacy Markdown, capsule rendering,
task closure and silent replacement of a database.

## Risk Classification

Risk flags: data model, durable records, existing behavior, weak proof.
Hard gate: data migration/rebuild safety. High-risk lane.

## Safety Invariants

- The retained foreign/ahead `harness.db` is never opened for write.
- Rebuild uses a sibling temp path and validates it before reporting success.
- Any future switch requires an explicit separate flag, backup and atomic
  rename; dry-run never creates a durable target.

## Validation

Use temporary repositories and databases to prove duplicate/conflict failure,
legacy/v1 projection, repeated rebuild equivalence and original-DB preservation.
