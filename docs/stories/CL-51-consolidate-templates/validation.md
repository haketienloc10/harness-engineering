# CL-51 Validation

`cargo test -p harness-cli` (63 tests), `cargo clippy -p harness-cli -- -D
warnings`, `bash tests/installer_state_safety.sh` and `git diff --check` pass.
