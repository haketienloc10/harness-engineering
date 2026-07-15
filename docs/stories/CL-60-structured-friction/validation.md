# CL-60 Validation

`cargo test -p harness-cli` (63 tests) and Clippy pass. Temporary-DB black-box
proof ran `init`, `friction add`, `friction resolve --status validated`, and
`friction query`, confirming the fingerprint and actual outcome persist.

The tiny task-finish unit fixture inserts linked material friction, proves the
gate rejects completion, marks it `validated`, then proves completion succeeds.
Observation-window reporting is owned by CL-61 maturity evidence.

2026-07-15 regression audit:

- `cargo test --workspace` passes all 63 tests and workspace Clippy passes.
- Source `cargo run -q -p harness-cli -- workflow parity --json` returns
  `WORKFLOW_PARITY_OK`.
- Packaged `_harness/bin/harness-cli workflow parity --json` returns
  `WORKFLOW_PARITY_DRIFT`; its compiled command list is missing `friction`,
  `friction add`, `friction resolve` and `friction query`.
- `bash tests/installer_state_safety.sh` consequently exits `1` at its packaged
  parity assertion.
- `cargo fmt --all -- --check` also fails on the newly added friction code.

Required before completion: format the source, rebuild the packaged binary from
the same source revision, and rerun format, Clippy, unit, command-parity and
installer black-box checks.
