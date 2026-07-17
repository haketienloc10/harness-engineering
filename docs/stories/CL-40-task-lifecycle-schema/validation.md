# CL-40 Validation

Domain tests must reject terminal bypasses and schema tests must reject invalid
status/terminal combinations. Legacy intake/trace links are never fabricated.

Passed: `cargo test -p harness-cli` covers the allowed transition graph and
SQLite terminal/status constraints. `task` command behavior and primary story
linking are validated separately by CL-41.
