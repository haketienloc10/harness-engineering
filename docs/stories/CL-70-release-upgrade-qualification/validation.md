# CL-70 Validation

## Result

Release qualification passes on the approved Linux `x86_64` scope. All mutable
installer, database, branch and session fixtures run in temporary directories;
the retained operational DB is never replaced, downgraded or edited manually.

`TASK-000013` supplies state qualification proof and `TASK-000014` supplies
distribution qualification proof. Both normal-lane dogfood tasks met context,
proof, standard-trace and capsule closure gates. Aggregate proof and terminal
observation are owned by high-risk `TASK-000012`.

## Environment Matrix

| Required environment | Evidence | Result |
| --- | --- | --- |
| Fresh Linux target | Local mocked archive installed to an empty Linux `x86_64` target | pass |
| Existing old-schema target | v1 behind-source detection, transactional `2..11` migration and backup fixture | pass |
| DB ahead of source | Doctor rejects v12 and proves the DB hash/version unchanged | pass |
| Branch-switch target | Candidate clone switches branch while preserving repository identity and healthy rebuilt DB | pass |
| Dirty worktree | Candidate clone remains healthy with an untracked dirty probe; proof freshness regression remains in the Rust suite | pass |
| CLI missing/wrong platform | Installer rejects a missing executable before target mutation and rejects mocked Darwin/arm64 | pass |
| Fresh clone + memory rebuild | Current candidate is cloned, committed in isolation, rebuilt twice to logical digest `958dec14e14bf88aff29ea5ba398012222fd7ea7c00ebbefc2c0db9a09f79e6d`, and strict doctor passes | pass |
| Installer rerun/upgrade | Repository ID, operational DB hash, product file, user ignore content and one managed block are preserved | pass |
| Two concurrent sessions | Second live session is rejected, then explicit block/reacquire/resume succeeds | pass |

## Release Evidence

| Evidence | Proof | Result |
| --- | --- | --- |
| Cargo format, Clippy, unit/integration | Full workspace ladder; 74 unit tests plus black-box integration tests | pass |
| Black-box CLI | Source and rebuilt packaged Phase 4 failure matrices | pass |
| Installer shell tests | Syntax plus fresh/rerun/upgrade/missing/wrong-platform state safety | pass |
| Migration/backup/restore | Behind, ahead, transactional migration, backup and restore fixtures | pass |
| Crash recovery | Source and packaged Phase 4 matrix plus staging/terminal rollback unit cases | pass |
| Payload manifest snapshot | Installed manifest/payload exclusions and retained-state checks | pass |
| AGENTS template parity | Installed shared block is byte-equal to canonical `AGENTS.md` | pass |
| Command manifest parity | Source, packaged and tracked manifests are equal at 73 commands | pass |
| Docs/DB roundtrip | Two isolated rebuilds have equal logical digest and strict healthy doctor result | pass |
| Startup latency | 30 packaged samples: p50 `2545 us`, p95 `3317 us`, below the approved `500000 us` p95 threshold | pass |
| Dogfood capsules | Eight completed capsules before aggregate closure; CL-70 closure produces the ninth | pass |

The installer now checks the extracted packaged CLI before writing any target
path. Its negative test proves an archive missing that executable leaves an
existing user file untouched and creates neither `AGENTS.md` nor `_harness/`.

## Observation Window

Before aggregate CL-70 closure, durable audit reports 12 evidence-backed
terminal tasks, tiny `4/3`, normal `4/4`, high-risk `4/2`, fresh-clone `1/1`,
installer-upgrade `2/1`, expanded closure gates `8/8`, and measured
improvements `2/2`. The only pre-closure gap is `blocked-resumed 0/1`.

`TASK-000012` was actually blocked to release the worktree for both normal
qualification tasks and then explicitly resumed. Its final trace records that
observed transition; terminal audit is the authority for the resulting H5
status and release-coverage promotion.

The three CL-70 tasks acknowledge all `8/8` stored must-read entries with zero
extra context acknowledgements; both terminal subtask proofs are passing and
fresh. There are zero human intervention records across CL-43, CL-61 and CL-70,
zero unexpected installer failures, zero memory-check errors, and two measured
friction outcomes. These counts complement the p50/p95 latency samples without
turning command availability into outcome evidence.

## Commands

- `cargo fmt --all -- --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `bash -n install.sh install-harness-cli.sh tests/installer_state_safety.sh tests/release_qualification.sh`
- `bash tests/release_qualification.sh all`
- `./install-harness-cli.sh`
- source and packaged `workflow parity --json`
- `memory check --dry-run --json`
- strict `doctor --json`
- `audit --json`
- `git diff --check`

## Rollback

Revert the installer preflight, audit coverage promotion, release script,
story/plan records and packaged binary together. Preserve operational DB,
proof outputs, traces, friction outcomes and task capsules. No schema rollback
or target-file restoration is required.
