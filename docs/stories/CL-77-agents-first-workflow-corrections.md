# CL-77 Agents-First Workflow Corrections

## Status

completed

## Lane and Scope

- Lane: high-risk
- Risk flags: public CLI contract, lifecycle gates, durable-state bootstrap
- In scope: record-free `init`, concrete task context, storyless proof/trace
  remediation, unambiguous capsule states, block reasons, CLI help, audit JSON
  semantics and collision-safe task IDs after portable-memory rebuild.
- Out of scope: a CLI file reader, interactive workflow wizard, new database
  schema, per-intake proof taxonomies and changes to strict audit thresholds.

## Product Contract

An agent can bootstrap durable state without creating a fake lifecycle root,
then follow task output whose context, remediation and state names describe
the concrete task. Read-only audit distinguishes actual debt from an empty
evidence window. Existing command-first ownership remains authoritative.

## Acceptance Criteria

- `harness-cli init` creates or safely migrates `harness.db` idempotently and
  reports that no lifecycle records were created.
- `task start` resolves changed files and a linked story artifact to concrete
  context paths; absent paths create no acknowledgement requirement.
- Storyless normal tasks use `task-validation`, and trace remediation omits a
  fake story argument.
- Required capsules remain `pending` until linked and valid; unrelated
  candidates are reported as orphans but never satisfy the capsule gate.
- `task block` requires a reason and `task status` exposes it.
- Help lists intake values and flag format without an interactive wizard.
- Non-strict audit reports incomplete coverage separately from consistency
  debt; strict behavior remains fail-closed.
- Task IDs remain unique across operational tasks and portable task summaries.

## Design and Decisions

Reuse the existing safe `ensure` path for `init` and the existing task outcome
column for block reasons. Derive UI contracts in the interface instead of
adding schema. Resolve context at task start/refresh from Git and
`artifact_index`; the CLI continues to let agents read files with their native
tools. Allocate task IDs from the maximum numeric ID in both durable sources.

ADR 0025 is amended only for record-free `init`. ADR 0026 remains the design
test: every emitted command and state should be deterministic and immediately
actionable by an agent.

## Human Gates

- User approval in the current session: implement the reviewed agents-first
  solutions and prefer simple designs where added complexity has little gain.

## Validation and Evidence

| Layer | Expected proof | Result |
| --- | --- | --- |
| Unit | `cargo test -p harness-cli` | pass: 83 tests |
| Integration | workflow parity and lifecycle contract tests | pass: parity; targeted lifecycle contracts in unit and installer fixtures |
| Platform | installer state-safety | pass |
| Release | packaged CLI command parity and live dogfood | pass: packaged `init`, parity, record-free idempotence and TASK-000041 allocation |

Additional validation: `cargo fmt --all -- --check`, `cargo clippy -p
harness-cli -- -D warnings`, shell syntax and `git diff --check` passed. The full
CLP/release scripts require external `jq`, which is unavailable in the current
environment; release qualification's obsolete `phase4_failure_matrix` test
target was corrected to the current unit test owner.

## Rollback and Harness Delta

Revert interface/domain behavior, tracked command manifest, packaged binary,
ADR amendment and this story together. `init` uses the existing atomic ensure
and backup behavior, so rollback does not require a schema rollback. Preserve
any created database or migration backup as operational evidence.
