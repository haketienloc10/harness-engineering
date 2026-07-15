# 0023 Portable Story Packet and Capsule Projection

Date: 2026-07-15

## Status

Accepted

## Context

Direct-child discovery skips story packets and nested task capsules, so a
fresh rebuild cannot reproduce the critical portable projection.

## Decision

- A story packet is identified by its directory and canonical `overview.md`.
- Allowed packet Markdown components are discovered in deterministic sorted
  repository-relative order and contribute to one aggregate checksum.
- Packet identity, title, status and lane come from `overview.md`; supporting
  components remain content of the same story rather than separate stories.
- Recursive discovery rejects symlinks, traversal, unsafe file types,
  duplicate IDs and case-colliding identities.
- A backward-compatible richer capsule schema may project only fields observed
  in durable records and immutable proof artifacts. v1 remains readable;
  unavailable legacy fields remain explicitly unknown.
- Dry-run rebuild is non-destructive. Retained-DB apply is backup-first,
  candidate-validated and atomic.

The user explicitly approved all CLP-001-R1 human gates on 2026-07-15;
`TASK-000025` stores the canonical approval records.

## Alternatives Considered

1. Treat every packet Markdown file as a story: rejected because it duplicates
   identity and loses packet semantics.
2. Introduce mandatory manifests immediately: rejected as unnecessary format
   churn for current canonical packets.
3. Invent missing historical capsule fields: rejected as fabricated evidence.

## Consequences

Rebuild must compare counts, IDs, paths, status, aggregate checksums, links and
critical proof summaries, and must report the actual candidate schema.

## Follow-Up

- Retain legacy single-file stories and v1 capsules through N+2.
- Apply to the retained DB only after strict candidate proof and backup.

