# CL-71 Validation and Evidence

## Proof Strategy

Evidence is accumulated by task and remains provisional until the terminal
qualification task proves the full checklist from a clean committed `HEAD`.
Every behavior-bearing proof is recorded with `harness-cli proof run` and a
story link. Read-only post-finish checks confirm terminal durable state.

## Required Proof Matrix

| Layer | Expected proof | Result |
| --- | --- | --- |
| Unit | `cargo test --workspace` | not run |
| Static | `cargo fmt --all -- --check`; `cargo clippy --workspace -- -D warnings`; `git diff --check` | not run |
| Integration | installer state safety; workflow parity; memory dry-run | not run |
| E2E | clean and dirty release qualification modes and fixtures | not run |
| Platform | `doctor --strict --json` is `HEALTHY` on Linux `x86_64` | not run |
| Audit | `audit --strict --json` has no unresolved or unknown coverage | not run |
| Release | `bash tests/release_qualification.sh all` from clean committed `HEAD` | not run |

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
- Clean committed `HEAD` proof and the full Task 1 closure ladder: pending.

## Terminal Evidence

Pending Tasks 1–3. Completion requires exact task ids, proof commands and
outcomes, approval sources, rollback procedure, compatibility obligation,
canonical backlog successor id, final post-finish doctor/audit output, and a
clean `git status --short`.
