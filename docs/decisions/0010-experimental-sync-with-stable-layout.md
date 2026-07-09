# 0010 Experimental Sync With Stable Layout

Date: 2026-07-09

## Status

Accepted

## Context

`harness-experimental` contains newer CLI, schema, installer, release, and
Symphony behavior. Its repository layout moved Harness runtime policy into
`docs/` and operational binaries and schema into `scripts/`. This repository
has an accepted public layout that keeps runtime files under `_harness/` and
product records under `docs/`.

Future updates need a durable way to identify the exact source revision without
replacing the accepted destination layout.

## Decision

Port behavior from `harness-experimental` while preserving these destination
paths:

- Runtime policy: `_harness/*.md`
- CLI: `_harness/bin/harness-cli`
- Schema: `_harness/scripts/schema/`
- Templates: `_harness/templates/`
- Product records: `docs/product/`, `docs/stories/`, `docs/decisions/`

Source references to `docs/<runtime-doc>`, `docs/templates/`,
`scripts/bin/harness-cli`, and `scripts/schema/` must be translated to their
`_harness/` equivalents. Repository branding and release URLs remain owned by
`harness-engineering`.

Each synchronization records the destination commit in
`harness-experimental` and the source commit in `harness-engineering` using
repository-local sync marker files. Build output and unrelated untracked source
files are excluded.

## Alternatives Considered

1. Replace the repository with the experimental layout. Rejected because the
   current `_harness/` contract must remain stable.
2. Merge Git histories. Rejected because the repositories do not share an
   ancestor and contain intentional structural differences.
3. Port only selected fixes without a baseline marker. Rejected because future
   incremental updates would not have a reliable comparison point.

## Consequences

Positive:

- Existing installed repositories keep stable paths.
- New experimental behavior can be adopted incrementally.
- Future syncs can diff from an explicit source and destination baseline.

Tradeoffs:

- Every sync needs deterministic path translation.
- Upstream tests and documentation must be adapted before their evidence is
  valid in this repository.

## Follow-Up

- Validate CLI, installer, schema upgrades, and Symphony using `_harness/`
  paths.
- Update both sync markers after commits are created.
