# CL-76 Installation Topology Modes

## Status

completed

## Lane and Scope

- Lane: high-risk
- Risk flags: architecture-direction, source-hierarchy, installer-state
- In scope: add `repository` and `coordination` installation modes, render
  mode-specific managed `AGENTS.md` guidance, and enforce root-only CLI use in
  coordination workspaces.
- Out of scope: native task scopes, cross-repository commit fingerprints, and
  automatic discovery or mutation of child repositories.

## Product Contract

`install.sh` remains a one-command installer. It defaults to `repository` mode
and accepts `HARNESS_INSTALL_MODE=coordination` for a Git-root workspace that
contains independent nested repositories. Coordination installations retain
user-authored `AGENTS.md` content, identify the root as the Harness control
plane, and forbid running the lifecycle CLI below that root.

## Acceptance Criteria

- Default installation remains byte-compatible in behavior with repository mode.
- Coordination mode rejects a target that is not its Git root.
- The installed manifest and generated installation config record the selected
  mode.
- The managed Harness block explains the active topology without overwriting
  user-authored workspace instructions.
- In coordination mode, `harness-cli` succeeds at the coordination root and
  rejects invocation from a nested repository, including an explicit
  `HARNESS_REPO_ROOT` override.

## Design and Decisions

Decision 0027 separates installation topology from workflow policy. A generated
`_harness/installation.toml` is installer-owned, while `_harness/workflow.toml`
remains the strict workflow-policy contract. The CLI reads the generated config
after resolving the root and checks its caller directory before operating.

## Human Gates

- `architecture-direction`: user explicitly requested both modes and root-only
  coordination behavior in this conversation.
- `source-hierarchy`: user explicitly requested preservation of their
  user-authored workspace `AGENTS.md` while adding generated topology guidance.

## Validation and Evidence

| Layer | Expected proof | Result |
| --- | --- | --- |
| Unit | `cargo test -p harness-cli` (81 tests) | pass |
| Integration | `bash tests/installer_state_safety.sh` mode fixtures | pass |
| Static | `bash -n install.sh`, `cargo fmt --check`, `git diff --check` | pass |
| Platform | `_harness/bin/harness-cli workflow parity --json` | pass |

`cargo clippy --workspace -- -D warnings` remains a known baseline failure on
pre-existing dead-code and `too_many_arguments` diagnostics outside this story's
scope; it was recorded as non-friction for TASK-000040.

## Rollback and Harness Delta

Revert the installer, generated topology config, CLI guard, and their tests as
one change. Existing installations without `_harness/installation.toml` remain
repository-mode compatible.
