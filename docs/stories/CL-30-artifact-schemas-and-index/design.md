# CL-30 Design

## Artifact Contract

V1 artifacts use YAML frontmatter with `schema`, `id`, `status`, and the
type-specific fields defined in CLP-001 section 10. Legacy heading files remain
readable and are reported as legacy rather than rewritten.

## Index Contract

`artifact_index` is a rebuildable projection keyed uniquely by artifact type,
semantic ID and repository-relative path. It stores content checksum, schema
version, status and source provenance; it never overrides Markdown truth.

## Boundary Rules

All artifact paths are normalized repo-relative paths. Traversal, absolute
paths, duplicate IDs/paths, unknown schema/status/lane and missing references
are deterministic validation failures. Checks use parsing only; index writes
are deferred to explicit CL-31 rebuild input.

## Alternatives

1. Parse only v1 frontmatter: rejected because current tracked records are
   legacy.
2. Rewrite legacy files during check: rejected because a read-only validation
   command must not create Git noise.
3. Treat the current DB as artifact authority: rejected by ADR 0011.
