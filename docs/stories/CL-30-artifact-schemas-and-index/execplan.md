# CL-30 Exec Plan: Artifact Schemas and Index

## Goal

Make Git-tracked stories, decisions and task capsules parseable semantic
artifacts, then expose a read-only `memory check --dry-run` projection without
rewriting documents or relying on the quarantined operational database.

## Scope

In scope: versioned frontmatter contracts, legacy heading parsers, validation,
safe repo-relative paths, duplicate/reference conflict reports, and a
rebuildable artifact-index schema/projection.

Out of scope: DB rebuild/switching (CL-31), capsule rendering (CL-32), task
lifecycle commands (CL-40+), and automatic Markdown conversion.

## Risk Classification

Risk flags: data model, public CLI contract, existing behavior, weak proof.

Hard gates: none. High-risk lane is selected because semantic records and the
durable index contract change together.

## Work Phases

1. Freeze legacy parser compatibility with fixtures.
2. Add artifact schema decision and migration `007` only after exact index
   fields are reviewed.
3. Implement pure parser/check commands with no document writes.
4. Project validated artifacts into a temporary/explicit DB only.
5. Run malformed, duplicate, missing-reference and legacy/v1 proof.

## Stop Conditions

Pause if schema/index ownership conflicts with the accepted rebuild or capsule
ADRs, or if an artifact requires a lossy legacy conversion.
