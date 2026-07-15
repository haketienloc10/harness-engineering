# CL-72 Execution Plan

## Sequence

1. CGR-00: bootstrap CL-72, freeze black-box gaps, record baselines and obtain
   the four named human approvals.
2. CGR-10: implement the lifecycle command and structured-error contract under
   its own normal behavior-bearing task.
3. CGR-20: implement approved recursive discovery and portable rebuild under
   its own high-risk task.
4. CGR-30: implement named audit coverage and truthful story/evidence
   reconciliation under its own high-risk task.
5. CGR-40: after CGR-10 through CGR-30 are committed and the worktree is clean,
   run terminal requalification under its own high-risk non-product task.

Only one live worktree lease may exist. Each behavior-bearing work item owns a
current proof, rooted trace, capsule, and lifecycle closure.

## Phase Gates

- CGR-00 finishes only after context acknowledgement, CL-72 registration and
  linkage, four packet files, reproduced negative fixtures, baseline counts and
  hashes, and truthful non-behavioral proof.
- CGR-10/20/30 do not start implementation before their named approvals.
- CGR-40 starts only from clean committed `HEAD` and owns the complete proof
  ladder, parent-plan amendments, detailed trace, capsule, and post-finish
  read-only checks.

## Rollback

Use the rollback boundaries in `CLP-001-R1` section 12. Never repair lifecycle
or artifact state through direct operational SQL writes.

