# CL-42 Exec Plan: Proof Run and Freshness

## Goal

Make proof a structured, append-only execution record tied to a task rather
than a mutable boolean claim.

## Scope

In scope: `proof run`, task/story link validation, structured executable/argv,
pass/fail append records, commit provenance and freshness derivation.

Out of scope: `task finish` closure, shell mode and raw-output persistence.

## Risk Classification

Risk flags: durable records, validation guarantees, existing behavior.

Hard gates: weakening validation is prohibited.

## Stop Conditions

Pause if fresh-proof semantics require a schema migration while the canonical
lineage cannot safely evolve with the retained ahead database.
