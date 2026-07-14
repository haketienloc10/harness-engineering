# CL-22 Policy Parity and Drift Gate

Status: ready

## Start State

- CL-21 pure context compiler is complete; task persistence remains owned by
  CL-40 and acknowledgement/refresh by CL-41.
- Root `AGENTS.md` is the canonical shared policy source and installer parity
  is already tested byte-for-byte.
- `_harness/command-manifest.txt` snapshots the Clap command tree and
  `workflow commands` renders the compiled tree.
- `_harness/tests/policy-parity-cases.toml` contains accepted classification and
  context comparison cases plus the one explicit unresolved semantic delta.
- `_harness/workflow.toml` is explicitly in `shadow` mode.

## CL-22 Scope

1. Parse the policy parity fixture and compare every accepted case with
   `workflow explain` or the pure context compiler.
2. Fail drift checks when root/installed shared policy, tracked/compiled command
   manifest, config cases or shadow output diverge.
3. Render intentional deltas separately from failures. The `one-flag-code-impact`
   delta must receive an explicit input-contract decision or ADR disposition;
   CL-22 must not silently choose one interpretation.
4. Keep Markdown policy authoritative and keep config in `shadow` mode.

## Safety Note For The Next Session

The retained local `harness.db` is intentionally the quarantined `001..008`
foreign/ahead recovery input. `doctor --strict` must continue returning exit
`3` with `DB_AHEAD_OF_SOURCE`; this is not a CL-22 failure. CL-22 is pure
policy/test work and must not run legacy intake, trace or other DB write
commands against that file. Use temporary fixture databases for tests and do
not rename, migrate or overwrite the retained database.

## Acceptance

- Every accepted fixture case passes in unit and packaged-binary black-box
  tests.
- Drift produces a stable non-zero result and identifies the surface/case.
- Intentional deltas name their disposition and approval/ADR reference.
- Installer test passes using `_harness/bin/harness-cli`, not only a Cargo-built
  test binary.

## Validation Commands

```bash
cargo test -p harness-cli
bash tests/installer_state_safety.sh
_harness/bin/harness-cli workflow validate --json
_harness/bin/harness-cli workflow commands
_harness/bin/harness-cli workflow context --lane normal --phase work --paths crates/harness-cli/src/interface.rs --json
```

## Rollback

Remove only CL-22 comparison/gate code and tests. Preserve the canonical
AGENTS source, typed compiler, fixtures and shadow policy for a later retry.
