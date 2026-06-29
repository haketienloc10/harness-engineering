# Test Matrix

This file maps Harness behavior to proof.

## Status Values

| Status      | Meaning                                        |
| ----------- | ---------------------------------------------- |
| planned     | Accepted as intended behavior, not implemented |
| in_progress | Actively being built                           |
| implemented | Implemented and proof exists                   |
| changed     | Contract changed after earlier implementation  |
| retired     | No longer part of the product contract         |

## Matrix

| Story | Contract                                | Unit | Integration | E2E | Platform | Status  | Evidence |
| ----- | --------------------------------------- | ---- | ----------- | --- | -------- | ------- | -------- |
| US-001 | Simple curl installer for `harness-engineering` | yes | yes | no | yes | implemented | `cargo test`; `bash -n install.sh`; temp install plus `_harness/bin/harness-cli init` and `query matrix` |
| US-002 | Harness runtime and docs paths | yes | yes | no | yes | implemented | `cargo test -p harness-cli`; `cargo build --release -p harness-cli`; `bash -n install.sh`; `git diff --check`; direct CLI help; local install smoke |
| US-003 | Workflow parity with experimental harness | yes | yes | no | yes | implemented | `cargo test -p harness-cli`; `bash -n install.sh`; `git diff --check`; CLI help/tool/matrix/audit checks; `story verify US-003` |

## Evidence Rules

- Unit proof covers pure domain and application rules.
- Integration proof covers backend enforcement, data integrity, provider
  behavior, jobs, or service contracts.
- E2E proof covers user-visible browser flows.
- Platform proof covers only shell, deployment, mobile, desktop, or runtime
  behavior that cannot be proven in lower layers.
- A story can be implemented without every proof column if the story packet
  explains why.
