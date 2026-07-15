# CL-72 Command Lifecycle and Portable-Memory Closure Corrections

## Status

in_progress

## Lane and Scope

- Lane: high-risk
- Risk flags: `architecture-direction`, `source-hierarchy`, `risk-policy`,
  `lifecycle-contract`, durable rebuild semantics, and retained database safety
- In scope: the complete `CLP-001-R1` contract in
  `_harness/docs/proposals/2026-07-15-clp-001-closure-gap-remediation-plan.md`
- Out of scope: platform expansion, destructive recovery, direct operational
  SQL writes, fabricated historical execution, validation weakening, and N+2
  compatibility removal

## Product Contract

The documented three-command lifecycle is executable and returns a complete,
stable contract. Git-tracked story packets and nested task capsules form a
safe, deterministic, rebuildable semantic projection. Strict audit coverage is
backed by named, current checks rather than inferred from a broad release
observation.

Historical tasks, traces, proofs, approvals, dispositions, and recovery
databases remain unchanged. Current reconciliation evidence may be added only
from observed durable records and current structured proof.

## Acceptance Criteria

- Every item in the `CLP-001-R1` terminal acceptance checklist has direct
  clean-HEAD evidence.
- `task start`, `task status`, and `task finish` satisfy their documented human,
  JSON, ownership, trace, error, and remediation contracts.
- Recursive artifact discovery validates packet stories and nested capsules,
  rejects unsafe or ambiguous inputs, and rebuilds the approved critical
  projection at the actual canonical schema.
- Strict audit reports named semantic-parity checks as `pass`, `fail`, or
  `unknown` with proof provenance and freshness.
- CL-01 and all retained CLP-001 work items have truthful canonical evidence or
  an explicitly approved visible disposition.
- Source, packaged, installed, and tracked CLI surfaces agree; installer and
  release qualification pass on Linux `x86_64`.

## Human Gates

Implementation is paused until the user explicitly approves:

- `architecture-direction`: story-packet identity and aggregate checksum and
  projection semantics;
- `source-hierarchy`: directory plus canonical `overview.md` versus a new
  manifest/component model;
- `risk-policy`: the boundary between proven and unknown audit coverage;
- `lifecycle-contract`: `behavior-bearing=auto` rules and final-trace ownership
  in `task finish`;
- any backup-first migration or rebuild applied to the retained operational DB.

## Validation and Evidence

The frozen black-box contracts are in `tests/clp001_r1_contracts.sh`. They are
run in `baseline` mode during CGR-00 to prove the gaps, and must pass in `all`
mode after remediation. Exact task IDs, proof hashes, counts, rollback evidence,
and compatibility obligations are recorded progressively in `validation.md`.

## Rollback and Harness Delta

Revert CLI, parser/index, audit-registry, packaged payload, manifests, tests,
and documentation as coherent units. Artifact or DB rollback is backup-first
and uses supported commands. Preserve all historical and remediation evidence.
