# 0009 Harness Runtime and Product Record Paths

Date: 2026-06-29

## Status

Accepted

## Context

The original payload used `.agent-harness/` for both agent runtime policy and
repo-specific product records. That made the target repository look like product
truth lived inside a hidden harness folder, even though product decisions,
product docs, and story packets are part of the target repo's product record.

The requested direction is to make the harness runtime explicit as `_harness/`
and keep product records under `docs/`.

## Decision

Install and operate the agent runtime from `_harness/`.

Use these product record directories in target repositories:

- `docs/product/`
- `docs/stories/`
- `docs/decisions/`

The CLI remains repository-local at `_harness/bin/harness-cli`. Schema
migrations remain under `_harness/scripts/schema/`. Templates remain under
`_harness/templates/`.

## Alternatives Considered

1. Keep all records under `.agent-harness/`. Rejected because product records
   should be visible product documentation in the target repo.
2. Move all harness files under `docs/`. Rejected because agent runtime policy,
   schema, templates, and CLI runtime are not product docs.

## Consequences

Positive:

- Target repos separate agent runtime from product records.
- Product docs, stories, and decisions become easier for humans and agents to
  inspect as repo documentation.
- The harness runtime path is no longer hidden.

Tradeoffs:

- Existing installed repos need the installer to back up legacy
  `.agent-harness/`.
- Documentation, tests, and CLI schema lookup paths must stay aligned with the
  new directory contract.

## Follow-Up

- Keep future product record templates pointed at `docs/`.
