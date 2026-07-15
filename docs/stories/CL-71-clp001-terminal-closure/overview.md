# CL-71 CLP-001 Terminal Closure and Historical Debt Reconciliation

## Status

in_progress

## Lane and Scope

- Lane: high-risk
- Risk flags: `architecture-direction`, `risk-policy`, durable schema migration,
  operational database migration, and historical audit acceptance
- In scope: clean/dirty release-suite reproducibility, current story evidence,
  explicit historical audit dispositions, legacy backlog `#4` provenance, and
  terminal CLP-001 qualification on Linux `x86_64`
- Out of scope: platform expansion, destructive database repair, direct
  operational SQL writes, validation weakening, early compatibility removal,
  credentials, and external cost

## Product Contract

CLP-001 reaches a truthful, machine-verifiable terminal state without
rewriting or fabricating historical records. Current product behavior and
durable state must agree; irreducible historical findings remain visible with
approved provenance but do not count as active audit debt.

Execution authority and the complete contract are in
`_harness/docs/proposals/2026-07-15-clp-001-full-closure-plan.md`.

## Acceptance Criteria

- The full terminal acceptance checklist in the execution authority is true.
- Clean committed `HEAD` passes release qualification.
- `doctor --strict --json` and `audit --strict --json` exit `0`.
- Canonical story status/evidence matches revalidated current contracts.
- Legacy backlog `#4` maps to a closed canonical successor with measured
  actual outcome and immutable recovery provenance.
- All required tasks have acknowledged context, approvals, fresh structured
  proof, truthful traces, capsules, released leases, and intentional commits.

## Affected Users

- Harness maintainers and agents relying on command-first lifecycle state.
- Reviewers who require audit findings to remain explicit and attributable.

## Non-Goals

- Retrospective execution claims or fabricated trace roots.
- Hiding accepted findings from human or JSON audit output.
- Broadening the approved Linux `x86_64` platform scope.

## Human Gates

- `architecture-direction`: approved by the user for `TASK-000020`; durable
  approval records the migration/CLI/disposition/backlog scope.
- `risk-policy`: approved by the user for `TASK-000020`; durable approval
  records strict-audit exclusions and expiry/revocation behavior.
- The same explicit approval covers the two named historical findings, legacy
  backlog `#4` successor mapping, and backup-first operational migration.
