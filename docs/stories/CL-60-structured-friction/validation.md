# CL-60 Validation

`cargo test -p harness-cli` (63 tests) and Clippy pass. Temporary-DB black-box
proof ran `init`, `friction add`, `friction resolve --status validated`, and
`friction query`, confirming the fingerprint and actual outcome persist.

The tiny task-finish unit fixture inserts linked material friction, proves the
gate rejects completion, marks it `validated`, then proves completion succeeds.
Observation-window reporting is owned by CL-61 maturity evidence.
