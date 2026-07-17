# 0024 Named Current Audit Coverage

Date: 2026-07-15

## Status

Accepted

## Context

Audit inferred semantic-memory parity from a broad release observation. This
could report coverage while packet stories and nested capsules were skipped.

## Decision

Every required audit coverage item is a named, versioned check with state
`pass|fail|unknown|not_applicable`, proof/command and artifact provenance,
HEAD/branch/dirty freshness, measured scope/counts, and remediation.

`semantic-memory-parity` passes only when recursive discovery, validation,
projection and fresh rebuild parity are all demonstrated at the actual
candidate schema. Missing, stale, failed, incomplete or count-mismatched proof
is `unknown` or `fail`. Strict audit rejects unresolved findings and every
required non-pass check.

The user explicitly approved all CLP-001-R1 human gates on 2026-07-15;
`TASK-000025` stores the `risk-policy` approval and scope.

## Alternatives Considered

1. Continue release-observation inference: rejected because it is indirect.
2. Treat missing parity as pass until a mismatch appears: rejected because
   absence of evidence is not evidence of parity.
3. Hide accepted historical dispositions: rejected; they remain visible and
   effective under their existing approved contract.

## Consequences

Strict audit will remain unknown until a current named semantic parity proof is
recorded. Negative fixtures must cover absent, stale and failed states.

## Follow-Up

- Reconcile CLP-001 story evidence without historical fabrication.
- Keep H5 derived from observed lifecycle evidence.

