# 0027 Installation Topology Modes

Date: 2026-07-17

## Status

Accepted

## Context

Harness previously treated every installation as one repository. A Git-root
workspace that coordinates several independent nested repositories needs one
lifecycle control plane without permitting lifecycle state in child repositories.

## Decision

`install.sh` supports two modes:

- `repository` is the default and preserves the existing single-repository
  contract.
- `coordination` is selected with `HARNESS_INSTALL_MODE=coordination` and is
  valid only when the target directory is a Git root.

The installer writes `_harness/installation.toml` and the manifest mode, then
renders a mode-specific managed topology section in `AGENTS.md`. It never
replaces user-owned instructions outside Harness markers.

In coordination mode, the CLI must be invoked from the coordination root. It
rejects calls from descendants, including calls that try to select the root via
`HARNESS_REPO_ROOT`. Workflow policy remains in `workflow.toml`; installation
topology is a separate runtime concern.

## Consequences

- A coordination root owns task lifecycle, integration records, proof and
  capsule artifacts.
- Nested Git repositories retain their own source, history, build, test and
  release flows.
- Native multi-repository task scope and child-repository fingerprints remain a
  separate future capability.
