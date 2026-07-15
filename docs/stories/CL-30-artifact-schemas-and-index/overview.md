# CL-30 Overview

## Current Behavior

Stories and decisions are heading-based Markdown, while the database stores
separate loose projections. No command can prove document identity, references
or a future capsule schema without writing operational state.

## Target Behavior

`story check`, `decision check` and `memory check --dry-run` validate both
legacy and v1 artifacts and emit deterministic conflict reports. Validated
artifact metadata can populate a rebuildable index; checks never rewrite docs.

## Status

completed

## Affected Product Docs

- `docs/stories/`
- `docs/decisions/`
- `docs/tasks/` (new capsule namespace)

## Non-Goals

- No automatic frontmatter migration.
- No mutation of the retained `harness.db`.
