# CL-70 Design

CL-70 adds one fail-fast distribution invariant and otherwise composes existing
contracts. `install.sh` validates that the extracted archive contains an
executable `_harness/bin/harness-cli` immediately after extraction and before
any target path is written.

`tests/release_qualification.sh` has independent `state` and `distribution`
modes. State qualification reuses the canonical Rust migration, backup/restore,
ahead-of-source, lease/session and crash-recovery tests, then creates a fresh
clone of the current worktree candidate. The clone proves deterministic memory
rebuild, capsule byte parity, branch switch, dirty worktree health, and two live
session contention with explicit block/resume recovery.

Distribution qualification runs installer fresh/rerun/upgrade and missing or
wrong-platform cases, compares source, packaged and tracked command manifests,
checks installed AGENTS/payload behavior, and measures thirty packaged CLI
startup samples. The initial Linux release threshold is p95 below 500 ms; the
measured p50/p95 remain evidence in proof output.

Operational `harness.db` is never copied into a release fixture. Every mutable
database and install target lives under a temporary directory and is deleted at
suite exit.
