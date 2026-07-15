# CL-71 Validation and Evidence

## Proof Strategy

Evidence is accumulated by task and remains provisional until the terminal
qualification task proves the full checklist from a clean committed `HEAD`.
Every behavior-bearing proof is recorded with `harness-cli proof run` and a
story link. Read-only post-finish checks confirm terminal durable state.

## Required Proof Matrix

| Layer | Expected proof | Result |
| --- | --- | --- |
| Unit | `cargo test --workspace` | 76 tests plus source Phase 4 pass during Task 2 implementation |
| Static | `cargo fmt --all -- --check`; `cargo clippy --workspace -- -D warnings`; `git diff --check` | Task 2 structured ladder passes |
| Integration | installer state safety; workflow parity; memory dry-run | Task 2 structured ladder passes |
| E2E | clean and dirty release qualification modes and fixtures | Task 1 dirty and clean proofs pass |
| Platform | `doctor --strict --json` is `HEALTHY` on Linux `x86_64` | source/operational migrations `001..012` are `HEALTHY` after backup-first migration |
| Audit | `audit --strict --json` has no unresolved or unknown coverage | Task 2 structured strict audit passes; accepted findings remain visible |
| Release | `bash tests/release_qualification.sh all` from clean committed `HEAD` | Task 1 clean proof passes; Task 2 dirty candidate proof passes; terminal clean rerun pending |

## Story Reconciliation Evidence Required

- CL-11 migration/backup/restore and strict doctor behavior.
- US-001 installer behavior.
- US-002 runtime, docs, and installation paths.
- US-003 workflow and command parity.
- US-005 structured friction query behavior.

Each retained story requires its own story-linked proof and truthful current
trace before status/evidence changes.

## Bootstrap Evidence

- Task: `TASK-000018`
- Context acknowledgement: `1/1` recorded by `codex`
- Proof command: `_harness/bin/harness-cli memory check --dry-run --json`
- Proof result: pass on `feature-rework` at
  `88f3e4e0932ae414921b2686f578726ec214bf0c`; the final fresh rerun is recorded
  in durable proof state before task finish
- Bootstrap trace: `#19` is detailed and meets the tiny-lane requirement; the
  final closure trace selected by `task finish` is `#20`
- Capsule: not required for this tiny task according to durable task state
- Finish result: `completed`; linked friction
  `9226055ccec105bf952f5c8f12417f28ec4ee547a16608b90efe64376770617b`
  was resolved as `validated`

## Release Reproducibility Evidence

- Task: `TASK-000019`
- Dirty candidate proof: layer `release-dirty` passed
  `bash tests/release_qualification.sh state` on `feature-rework` at
  `88f3e4e0932ae414921b2686f578726ec214bf0c`.
- Candidate composition: tracked change in
  `tests/release_qualification.sh` plus untracked proposal and CL-71 artifacts;
  this proves both tracked-patch application and untracked-file copying.
- Dirty proof stdout hash:
  `49385643d66918cbdd18246c5faa7877d288d57f082f2a5225c147850dd898dd`.
- Clean committed `HEAD` proof at `62338bd76280fb78b17a0fb8970991794be85cb9`
  passed format, Clippy, 74 workspace tests, shell syntax, `state`,
  `distribution`, `all`, and `git diff --check` through structured proof.
- Clean `release-all` stdout hash:
  `89326512252a20560e9ada64020244b3e152d427a15f315504b4a548d5810d7e`.
- Closure trace: `#21`; CLI-rendered capsule:
  `docs/tasks/2026/07/TASK-000019-made-release-qualification-reproducible-for-clean.md`.

## Durable Reconciliation Evidence

- Task: `TASK-000020`; context `4/4`; durable approvals:
  `architecture-direction` and `risk-policy` from the explicit user approval
  of items 1–6.
- Story proofs/traces: CL-11 `story-cl11`/`#22`, US-001
  `story-us001`/`#23`, US-002 `story-us002`/`#24`, US-003
  `story-us003`/`#25`, and US-005 `story-us005`/`#26`; all proofs pass and all
  traces meet detailed tier.
- Canonical migration `012-audit-disposition.sql` passed 76 workspace tests.
  Operational migration moved `001..011` to `001..012` through the packaged
  backup-first CLI. Pre-migration DB SHA-256 was
  `2f062c0cecad5666bb436dd49389f270f7626d701beaca45afa76579c51507cc`;
  backup path is
  `harness.db.backups/harness.db.1784104246235156032.v11.main.a5af5ab9060c.bak`.
- Disposition `#1` accepts only `terminal_task_without_trace:TASK-000006` and
  cites replacement `TASK-000007` proof at `3d6b68d`; disposition `#2` accepts
  only `unrooted_trace:2` and cites `TASK-000002` proof at `0df8291`.
- Both findings remain visible under `accepted_findings`; unresolved and
  unknown coverage are empty. Revoked or expired rows are regression-proven to
  re-open debt; `unknown_coverage` acceptance is rejected.
- CL-11 and US-001/002/003/005 durable statuses are `implemented` from their
  current story-linked proofs.
- Legacy backlog `#4` recovery sources and checksums map to open canonical
  successor `#2`, created through `harness-cli backlog add`; it remains open
  until terminal qualification.
- Task 2 initial structured proof ladder passed format, Clippy, 75 workspace
  tests, installer safety, full release qualification, workflow parity, memory
  dry-run, strict doctor, strict audit and `git diff --check`. A final
  fail-closed doctor contract test raised the suite to 76; its full clean-HEAD
  structured rerun is recorded after the implementation commit.
- Initial proof hashes: unit stdout
  `818dc59b3f3f0380785bb11b37184a2c2e74145f38db8eebb1b8db7c5e119b8d`;
  release stdout
  `7e479dde1fa63674f8180f58bad851ddc86b87f54966a0549efe22c15b209fbe`;
  doctor stdout
  `a356c704fe556eb49b9dcfc52e30e2a29fcc3a40b4eac2716f85be71fa8eb97e`;
  audit stdout
  `058ff899e1e1d0d8be17bdbc76f4be6e119bff6679f6f9a09d4629fdb8d4bba4`.

## Terminal Evidence

Pending Tasks 1–3. Completion requires exact task ids, proof commands and
outcomes, approval sources, rollback procedure, compatibility obligation,
canonical backlog successor id, final post-finish doctor/audit output, and a
clean `git status --short`.
