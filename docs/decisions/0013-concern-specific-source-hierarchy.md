# 0013 Concern-Specific Source Hierarchy

Date: 2026-07-14

## Status

Accepted

## Decision

Authority is per concern: user instructions own current intent; product,
story, and decision Markdown own semantic records; `workflow.toml` and
accepted ADRs own Harness policy; current proof runs own execution evidence;
task capsules own portable memory. Editable matrices are not authoritative and
are replaced by derived views.

## Consequences

CLI projections may report inconsistency but never override semantic sources.
