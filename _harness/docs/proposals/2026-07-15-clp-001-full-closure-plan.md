# CLP-001 Full Closure Plan

Date: 2026-07-15

Status: Fully closed — Tasks `TASK-000018..21` supply bootstrap,
release-reproducibility, durable reconciliation and terminal qualification
evidence on Linux `x86_64`

Parent plan:
`_harness/docs/proposals/2026-07-14-command-first-lifecycle-execution-plan.md`

## 1. Objective

Move CLP-001 from `Release-qualified with historical debt` to a truthful,
machine-verifiable terminal state without rewriting or fabricating historical
records.

The execution is complete only when:

- the release qualification suite passes from a clean committed HEAD;
- all canonical story records are supported by current evidence;
- every remaining audit finding is either resolved or explicitly accepted
  through a durable, approved disposition;
- legacy backlog `#4` has a provenance-preserving canonical successor with a
  measured actual outcome;
- `doctor --strict` and `audit --strict` pass;
- the final lifecycle task is completed with context, approval, structured
  proof, trace, and capsule gates satisfied.

Approved platform scope remains Linux `x86_64`. Platform expansion,
destructive database repair, direct operational SQL, validation weakening, or
early compatibility removal is outside this plan.

## 2. Verified Starting State

The 2026-07-15 review established:

- all 22 CLP-001 work items are marked completed;
- CL-70 and H5 are achieved on Linux `x86_64`;
- source and operational database migrations are both `001..011`;
- `doctor --strict --json` reports `HEALTHY`;
- format, workspace Clippy, 74 unit tests, source/packaged Phase 4 matrices,
  workflow parity, memory dry-run, installer safety, and distribution
  qualification pass;
- `audit --strict --json` exits `6` with historical `AUDIT_DEBT`;
- `bash tests/release_qualification.sh all` exits `128` on a clean worktree
  because an empty patch is passed to `git apply`;
- there are no active lifecycle tasks before the plan-artifact task is started.

Current unresolved audit findings are:

- orphaned planned stories `US-001`, `US-002`, `US-003`, and `US-005`;
- abandoned `TASK-000006` without a trace;
- trace `#2` without an intake/task root.

Durable-state drift also includes CL-11 remaining `planned` in the story table
while the parent execution plan records it as completed.

## 3. Recovered Backlog Provenance

Legacy backlog `#4` is recoverable from retained read-only backups. It is not
necessary to invent its prior content.

Verified sources include:

- `.harness-backup/cl-00-20260714T000000+0700/harness.db`;
- `harness.db.backups/harness.db.1784079667312317714.v8.main.c1cf74965c79.bak`;
- `harness.db.backups/harness.db.1784079894588525152.v8.main.c1cf74965c79.bak`;
- `harness.db.backups/rebuild-607656.db`.

The retained record is:

- legacy id: `4`;
- title: `Establish schema-safe closed-loop Harness lifecycle`;
- status: `proposed`;
- predicted impact: `Unsafe schema/branch state fails before operational
  queries and later task closure becomes enforceable from trustworthy state`;
- actual outcome: absent in the historical database;
- notes: the proposal is version-controlled and the mixed Symphony database
  must be preserved/exported before repair.

Do not insert id `4` with manual SQL. Create a canonical successor through
`harness-cli backlog add`, record the legacy id, source backup path and checksum
in its notes, then close that successor only after terminal qualification.
Amend the parent plan to map legacy backlog `#4` to the new canonical id.

## 4. Lifecycle Strategy

Create one new story:

`CL-71 — CLP-001 terminal closure and historical debt reconciliation`

Because high-risk tasks require an existing story, use a small non-behavioral
bootstrap task to register CL-71 before beginning behavior-bearing work. Every
repository or durable-state change must occur under an active lifecycle task.

Recommended task sequence:

| Order | Task | Lane | Primary responsibility |
| ---: | --- | --- | --- |
| 0 | CL-71 bootstrap | tiny | Register the story and its progressive artifacts |
| 1 | Release-suite reproducibility | normal | Make clean and dirty candidate qualification repeatable |
| 2 | Durable-state and audit reconciliation | high-risk | Reconcile stories, add explicit audit dispositions, restore backlog provenance |
| 3 | Terminal plan qualification | high-risk | Prove clean committed HEAD and close CLP-001 |

Run the tasks sequentially in one worktree. Do not retain two live worktree
leases at once. Use explicit, unique owner/session pairs.

## 5. Task 0 — Bootstrap CL-71

Start a tiny, non-behavior-bearing task before adding the story:

```bash
_harness/bin/harness-cli task start \
  --type "harness improvement" \
  --summary "Register CL-71 terminal closure story" \
  --owner codex \
  --session cl71-bootstrap-<timestamp> \
  --flags docs \
  --behavior-bearing no \
  --json
```

Then:

1. Read and acknowledge the returned context manifest.
2. Add CL-71 through `harness-cli story add`; do not write DB rows directly.
3. Create `overview.md`, `design.md`, `execplan.md`, and `validation.md` under
   `docs/stories/CL-71-clp001-terminal-closure/`.
4. Link CL-71 to the bootstrap task if supported by the resulting lane/gates.
5. Run a quick `memory check --dry-run --json` through `proof run`.
6. Record a truthful trace and finish the task, generating its capsule when
   required.

## 6. Task 1 — Release-Suite Reproducibility

### Scope

Fix `tests/release_qualification.sh` so the fresh-clone candidate path supports
both clean and dirty source worktrees.

Required behavior:

- materialize `git diff --binary HEAD` into a temporary patch;
- run `git apply` only when that patch is non-empty;
- copy untracked candidate files as today;
- commit candidate changes only when the clone has staged changes, or otherwise
  deliberately retain the existing cloned HEAD;
- preserve branch-switch, dirty-worktree, memory-rebuild, capsule, and session
  fixtures.

### Regression matrix

- clean committed HEAD;
- tracked dirty candidate change;
- untracked candidate file;
- source and packaged Phase 4 matrices;
- state-only, distribution-only, and `all` modes.

### Closure gate

At minimum, structured proof must include:

```text
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
bash -n tests/release_qualification.sh
bash tests/release_qualification.sh state
bash tests/release_qualification.sh distribution
bash tests/release_qualification.sh all
git diff --check
```

Finish the task only after its proof is passing and fresh. Commit this change
before the final clean-HEAD qualification task.

## 7. Task 2 — Durable-State and Audit Reconciliation

This task is high-risk because it changes audit semantics and durable state.
Record explicit human approval for `architecture-direction` and `risk-policy`
before implementation or closure.

### 7.1 Story reconciliation

Revalidate current contracts before changing status:

- CL-11 migration/backup/restore and strict doctor behavior;
- US-001 installer behavior;
- US-002 runtime/docs/install paths;
- US-003 workflow and command parity;
- US-005 structured friction query behavior.

For every retained canonical story:

- link it to the active reconciliation task as primary or secondary;
- run a story-linked structured proof;
- record a trace with the real current intake and story id;
- update status/evidence only after proof passes;
- use `retired`, rather than `implemented`, if the contract is no longer a
  canonical product surface.

A single task may link multiple stories, but each story must receive its own
truthful trace and proof linkage. Do not create a trace that claims historical
execution which was not observed.

### 7.2 Explicit historical audit disposition

Do not hide `TASK-000006` or trace `#2` by fabricating retrospective execution.
Add a machine-readable audit-disposition contract instead.

Recommended contract:

- a canonical schema migration adds durable audit dispositions;
- each disposition identifies the exact finding key and entity;
- fields include status, rationale, provenance, approval source, actor,
  creation time, and optional expiry/revocation;
- CLI supports add/list/revoke through a command-first write path;
- human and JSON audit output keep accepted findings visible separately;
- `audit --strict` passes only when there are no unresolved findings or unknown
  coverage;
- expired or revoked dispositions become unresolved again;
- acceptance is forbidden for health failures, unknown coverage, destructive
  recovery, or weakened validation.

Accept only the two irreducible historical findings:

- `TASK-000006` lacks an original trace because it was abandoned and replaced
  after discovering high-risk schema/CLI work;
- trace `#2` is unrooted, but its persisted notes and actions identify the
  existing `TASK-000002` proof at commit `0df8291`.

Both acceptances must cite immutable evidence and the human approval. They must
remain visible in audit output and must not count as active debt.

### 7.3 Backlog #4 canonical successor

1. Record checksums for the source recovery databases.
2. Read the legacy record through a read-only CLI path.
3. Add a canonical successor with the original title and predicted impact.
4. Include legacy id `4`, source path/checksum, and CLP-001 mapping in notes.
5. Keep it open until final qualification passes.
6. Close it with a measured actual outcome covering doctor health, release
   qualification, H5, strict audit, and task closure.

### Task 2 proof ladder

```text
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
bash tests/installer_state_safety.sh
bash tests/release_qualification.sh all
_harness/bin/harness-cli workflow parity --json
_harness/bin/harness-cli memory check --dry-run --json
_harness/bin/harness-cli doctor --strict --json
_harness/bin/harness-cli audit --strict --json
git diff --check
```

Update CL-71 validation evidence and the parent plan before finishing this task.
Commit the source, migration, packaged binary, story, plan, and capsule changes
together as required by the relevant rollback contract.

## 8. Task 3 — Terminal Plan Qualification

Start this task only after Task 2 changes are committed and the worktree is
clean. It owns no new product behavior; it proves and records the terminal
state.

Required structured proofs:

```bash
_harness/bin/harness-cli proof run --task <TASK_ID> --story CL-71 \
  --layer release -- bash tests/release_qualification.sh all

_harness/bin/harness-cli proof run --task <TASK_ID> --story CL-71 \
  --layer audit -- _harness/bin/harness-cli audit --strict --json

_harness/bin/harness-cli proof run --task <TASK_ID> --story CL-71 \
  --layer platform -- _harness/bin/harness-cli doctor --strict --json
```

Also run format, workspace Clippy/tests, workflow parity, memory dry-run, shell
syntax, installer safety, and `git diff --check` through structured proofs or a
single declared release proof command.

Before `task finish`:

- acknowledge every stored must-read context entry;
- record the required high-risk approval;
- confirm all required proofs are passing and fresh;
- update CL-71 validation and the parent plan with exact task ids, commands,
  outcomes, rollback procedure, remaining compatibility obligation, and the
  canonical backlog successor id;
- record a detailed trace rooted in the task intake;
- record and resolve any structured friction discovered during execution.

Then close through `harness-cli task finish`; do not manually set terminal DB
state or hand-write a capsule.

## 9. Terminal Acceptance Checklist

CLP-001 may be marked `Fully closed` only when every item is true:

- [x] All original 22 work items remain completed.
- [x] CL-71 is implemented with durable validation evidence.
- [x] No lifecycle task remains active after terminal closure.
- [x] Final task status is `completed` with released lease.
- [x] Final context acknowledgement count equals the required count.
- [x] Required high-risk approvals are present.
- [x] Required proof runs pass with output/artifact provenance.
- [x] Every required normal/high-risk task has a valid capsule and trace.
- [x] `doctor --strict --json` exits `0` with `HEALTHY`.
- [x] `audit --strict --json` exits `0` with no unresolved findings and no
      unknown coverage.
- [x] Accepted historical findings remain explicitly visible with approval and
      provenance.
- [x] H5 remains `achieved` from observed lifecycle evidence.
- [x] The full release suite passes from a clean committed HEAD.
- [x] Source, packaged, and tracked command manifests agree.
- [x] Memory rebuild/check and installer state-safety pass.
- [x] CL-11 and retained US story statuses match their verified contracts.
- [x] Legacy backlog `#4` maps to a closed canonical successor with measured
      actual outcome.
- [x] The parent plan records the final evidence, rollback, and compatibility
      window obligation.
- [x] Post-finish read-only doctor/audit checks pass.
- [x] Repository changes and generated capsules are committed intentionally;
      final `git status --short` is clean.

## 10. Human Gates and Stop Conditions

Pause and request approval before:

- accepting the audit-disposition semantics;
- accepting either historical finding;
- mapping legacy backlog `#4` to a differently numbered canonical successor;
- changing source hierarchy, lifecycle completion invariants, or risk policy;
- applying a schema migration to the retained operational database;
- any destructive recovery, manual SQL, validation weakening, platform
  expansion, compatibility removal, credential use, or external cost.

The instruction to execute this plan in a future session should be recorded as
direction approval only if the user explicitly authorizes implementation, not
merely planning or review.

## 11. Rollback

- Release script: revert the conditional candidate-patch/commit change.
- Audit disposition: restore the pre-migration database backup with the prior
  packaged binary; never drop or rewrite live columns manually.
- Story/backlog reconciliation: preserve the historical backups and revert
  Markdown/source changes; use lifecycle commands for durable-state correction.
- Final closure: preserve traces, proofs, friction records, approvals, and task
  capsules as historical evidence even if product code is reverted.

## 12. New-Session Start Prompt

Suggested instruction for the next session:

> Execute
> `_harness/docs/proposals/2026-07-15-clp-001-full-closure-plan.md`
> sequentially. Start with the required command-first lifecycle task, stop for
> every listed human gate, do not fabricate historical records, use structured
> proof runs, and do not finish until the terminal acceptance checklist passes.

## 13. Corrective Amendment — CLP-001-R1

The `Fully closed` history above remains preserved. Independent audit
`TASK-000022` later found three literal closure gaps: the documented minimal
`task start/status/finish` path did not match the CLI contract; recursive
semantic memory skipped packet stories and nested capsules; and strict audit
inferred semantic coverage from a broad release observation. The approved
`CLP-001-R1` remediation closed those gaps through tasks `TASK-000024..28`.

Current correction evidence:

- CLI `0.1.12` implements typed conservative `behavior-bearing=auto`, complete
  lifecycle results/remediation, stable JSON errors, and truthful exactly-one
  trace auto-selection for minimal finish.
- Canonical schema `13` and the portable projection rebuild recursive packet
  stories and nested v1/v2 capsules from deterministic safe discovery, with
  actual-schema reporting, complete parity metadata, and backup-first atomic
  retained apply.
- Five named current audit checks cover Markdown/DB fields, path-scoped proof
  freshness, generated matrix/CLI payload parity, semantic memory parity, and
  operational telemetry. Missing, failed, incomplete, or stale evidence no
  longer counts as covered.
- CL-01 and CL-72 provide truthful current canonical evidence while all
  historical lifecycle records and the two approved dispositions remain
  unchanged and visible.
- `TASK-000028` owns the clean-HEAD terminal proof, detailed rooted trace,
  CLI-rendered capsule, atomic finish, and post-finish read-only task, doctor,
  audit, memory, capsule, and Git checks on Linux `x86_64`.

Rollback must revert source, packaged binary, migration, tests, and documents
as a coherent unit and restore retained state only from a validated backup with
the matching prior binary. Preserve all task/proof/trace/approval/disposition/
friction/capsule evidence. The existing N+2 compatibility obligation and
platform boundary are unchanged.
