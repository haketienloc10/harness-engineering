# CL-71 Execution Plan

## Goal

Execute CLP-001 sequentially until the repository and durable Harness state
meet every terminal acceptance criterion.

## Task Sequence

1. `TASK-000018`: bootstrap CL-71 and progressive artifacts under a tiny,
   non-behavior-bearing task.
2. `TASK-000019`: release-suite reproducibility under a normal
   behavior-bearing task.
3. Durable-state and audit reconciliation under a high-risk task.
4. Terminal plan qualification from a clean committed `HEAD` under a high-risk
   task that owns no new product behavior.

Only one live worktree lease may exist at a time. Every repository or durable
state change occurs under its active task.

## Required Human Gates

Pause before:

- accepting audit-disposition semantics or either historical finding;
- mapping legacy backlog `#4` to a differently numbered successor;
- changing source hierarchy, lifecycle completion invariants, or risk policy;
- applying a schema migration to the retained operational database;
- destructive recovery, manual SQL writes, validation weakening, platform
  expansion, compatibility removal, credentials, or external cost.

Task 2 requires recorded `architecture-direction` and `risk-policy` approval.
Task 3 requires its policy-selected high-risk approval before finish.

## Phase Gates

- Task 0: context `1/1`, CL-71 linked, four artifacts present, memory dry-run
  proof passes, truthful trace recorded, task finishes through the CLI.
- Task 1: the complete clean/dirty regression matrix and closure proof ladder
  pass; changes and capsule are committed before Task 2.
- Task 2: story-specific current proofs pass, dispositions and backlog
  successor are approved and durable, strict audit passes, parent plan and
  validation are current, and all source/state/package changes are committed.
- Task 3: all terminal proofs pass from clean committed `HEAD`, every checklist
  item has direct evidence, the final task finishes, and post-finish read-only
  doctor/audit checks pass with a clean worktree.

## Rollback

- Revert the conditional release candidate patch/commit change.
- Restore the pre-migration operational database backup with the prior packaged
  binary; never drop or rewrite live columns manually.
- Preserve historical backups and revert Markdown/source changes; correct
  durable story/backlog state only through lifecycle commands.
- Preserve traces, proofs, friction, approvals, and capsules as historical
  evidence even when product code is reverted.
