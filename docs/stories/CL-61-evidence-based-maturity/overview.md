# CL-61 Evidence-based Audit and Maturity

## Status

completed

## Lane and Scope

- Lane: high-risk
- Risk flags: audit contract, lifecycle evidence, installed CLI parity
- In scope: audit debt/coverage, outcome-derived H5 assessment, dogfood
  observation counts, human/JSON/strict CLI modes, source and packaged proof.
- Out of scope: closing reported debt, claiming H5 while observation gaps remain,
  and CL-70 release qualification.

## Product Contract

`audit` is a read-only consistency and maturity report. It does not substitute
for `doctor`: every report says health was not checked and directs callers to
`doctor` for repository/database health.

The report distinguishes checked coverage from unknown coverage. Human and
JSON output are rendered from the same `AuditResult`; `audit --strict` exits `6`
while either findings or unknown coverage remain.

H5 is derived from observed lifecycle evidence, never command existence. The
assessment requires all of the following:

- at least ten terminal tasks with a trace linked through the task intake;
- at least three tiny, four normal and two high-risk observed tasks;
- at least one trace action marker for each of `blocked-resumed`,
  `fresh-clone-rebuild` and `installer-upgrade`;
- every completed normal/high-risk task satisfies persisted closure gates;
- at least two terminal friction improvements with non-empty `baseline`,
  `predicted_metric`, `observation_window` and `actual_outcome`.

## Acceptance Criteria

- `audit --json` returns stable debt, checked/unknown coverage and maturity
  fields from one domain result.
- `audit --strict` exits `6` on current debt/unknown coverage without mutating
  state.
- Terminal task/trace linkage, unrooted traces and completed expanded-task gate
  debt are audited explicitly.
- H5 remains `not_achieved` until the measured-improvement threshold and the
  complete dogfood observation window are both met.
- Source and packaged CLI behavior, workflow parity and installer state safety
  pass.

## Design and Decisions

The implementation adds no migration and no write path. SQL derives counts from
existing `task`, `trace`, `proof_run`, `task_context_read`, `task_approval` and
`friction` records. Exact trace action markers provide deterministic scenario
evidence without guessing from summaries or notes.

Decision 0005 remains authoritative: the packaged repository-local Rust binary
is rebuilt from the same source and retains `_harness/bin/harness-cli` as the
stable command path.

## Human Gates

The explicit user instruction approving CL-61 implementation is recorded on
`TASK-000011` as the high-risk direction approval.

## Validation and Evidence

See `validation.md`. The outcome-derived snapshot has two measured
improvements, but H5 correctly remains `not_achieved` because the traced task
mix and three scenario markers are incomplete.

## Rollback and Harness Delta

Rollback the three Rust source changes, this story update, the maturity report,
the plan reconciliation and the rebuilt packaged binary together. No schema or
operational database rollback is required. Preserve task, proof, trace and
friction records as historical evidence.
