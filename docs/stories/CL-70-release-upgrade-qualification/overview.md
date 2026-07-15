# CL-70 Release/Upgrade Qualification

## Status

completed

## Lane and Scope

- Lane: high-risk
- Risk flags: release, upgrade, retained state, recovery, distribution parity
- In scope: the Phase 7 release environments and evidence in CLP-001, the
  remaining H5 dogfood window, repeatable source/packaged qualification, and
  fail-fast handling of a missing packaged CLI.
- Out of scope: adding another target platform, weakening a validation gate,
  destructive recovery of the retained operational database, or removing a
  compatibility surface before its approved release window.

## Product Contract

The release candidate is qualified only when source and packaged behavior agree,
fresh and upgrade installs preserve user/product state, migration and recovery
fixtures pass, a fresh clone can rebuild canonical memory, dirty/branch/session
environments remain safe, and the observed H5 task window meets its thresholds.

An installer archive without the Linux `x86_64` packaged CLI must fail before it
mutates the target. Command availability alone is not release or H5 evidence.

## Acceptance Criteria

- Every environment listed by CL-70 has repeatable evidence or an explicit,
  approved gap.
- Format, Clippy, workspace tests, source/packaged black-box tests, installer
  safety, migration backup/restore, crash recovery, payload/AGENTS/command
  parity, docs/DB rebuild and startup latency pass.
- The release suite runs against the current worktree candidate and uses only
  isolated temporary operational state.
- Five to ten tracked task capsules exist after closure.
- `audit --json` reports H5 achieved from task-linked evidence and measured
  outcomes, not from fixture-only command existence.

## Human Gates

`TASK-000012` records the user's explicit high-risk direction approval. Any
destructive recovery, platform expansion or validation weakening remains a new
stop condition requiring separate approval.

## Validation and Evidence

See `validation.md`. State and distribution dogfood tasks `TASK-000013` and
`TASK-000014` passed with structured proof and capsules. `TASK-000012` owns the
aggregate release proof and terminal H5 observation.

## Rollback and Harness Delta

Rollback the installer preflight, release qualification script, story/plan
reconciliation and packaged CLI together. Preserve operational DB, proof,
trace, friction and task-capsule evidence; no schema migration is introduced.
