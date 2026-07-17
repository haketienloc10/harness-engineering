# 0010 Main Schema Lineage Without SYMPHONY

Date: 2026-07-14

## Status

Accepted

## Context

The baseline source commit contains canonical migrations `001..005`, while the
local ignored `harness.db` reports versions `001..008`. Migrations `006..008`,
story `US-004`, and an unrelated experimental decision were traced to the
separate `SYMPHONY` branch. They are not part of `main`.

The command-first lifecycle requires a deterministic, checksum-verified
migration lineage. Treating a higher local version as automatically current
would let a foreign branch silently redefine the main schema and product
memory.

## Decision

Select Case B from CLP-001.

- `main` migrations `001..005` are the canonical lineage at the baseline.
- A database containing `006..008` from `SYMPHONY` is foreign/ahead and must
  fail closed; it is never downgraded in place.
- Recovery uses a separately retained snapshot and an explicit rebuild from
  canonical Git artifacts, not a silent reset.
- The next canonical migration is `006-*` only after the migration manifest,
  checksum algorithm, and safe ensure/migrate path are implemented.
- `US-004` and the experimental `SYMPHONY` decision are excluded from a main
  memory rebuild unless a later reviewed import explicitly adopts them.

## Alternatives Considered

1. Accept `SYMPHONY` migrations as main (Case A). Rejected because their
   artifacts are not in `main` and would import unrelated branch semantics.
2. Keep version `8` as current because it is numerically newer. Rejected:
   monotonically larger versions do not prove compatible lineage.
3. Delete or downgrade the local DB. Rejected because it risks durable data and
   destroys recovery evidence.

## Consequences

`doctor` and `ensure` must distinguish source lineage from a bare schema
version. Until CL-10 and CL-11 exist, the current DB remains a recovery input
only; no normal lifecycle command may declare it healthy for the new system.

## Follow-Up

- Approve the remaining CL-01 ADRs before CL-10.
- Create the migration manifest and implement foreign-lineage detection.
- Add an explicit reviewed rebuild/import path for any future adoption of
  `SYMPHONY` artifacts.
