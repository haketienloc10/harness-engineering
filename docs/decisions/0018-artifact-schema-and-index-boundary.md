# 0018 Artifact Schema and Index Boundary

Date: 2026-07-14

## Status

Accepted

## Decision

Stories, decisions and task capsules are Git-tracked semantic artifacts. New
artifacts use versioned YAML frontmatter; existing heading-based records remain
supported as legacy input. Validation is read-only and reports legacy state,
schema errors, duplicate identity/path and missing references without rewriting
documents.

`artifact_index` is a rebuildable SQLite projection containing artifact type,
semantic ID, repo-relative path, checksum, schema version, status and source
provenance. It is not a source of truth and cannot repair or override a
Markdown artifact. Index writes belong to the explicit rebuild path, never to
`memory check --dry-run`.

## Consequences

CL-30 can ship parsers and a deterministic validation contract without touching
the quarantine DB. CL-31 owns temporary-DB construction, conflict-safe rebuild
and the explicit atomic switch.
