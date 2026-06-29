# US-003 Workflow Parity With Experimental Harness

## Status

implemented

## Lane

normal

## Product Contract

The `harness-engineering` payload keeps its `_harness/` runtime layout while
carrying the operational workflow guidance proven in `../harness-experimental`:
story verification, fresh tool presence checks, component/maturity/audit
documentation, proposal workflow, synchronized fallback proof, and durable
decision records.

## Relevant Product Docs

- `AGENTS.md`
- `_harness/HARNESS.md`
- `_harness/CONTEXT_RULES.md`
- `_harness/README.md`
- `_harness/TEST_MATRIX.md`
- `_harness/HARNESS_AUDIT.md`
- `_harness/HARNESS_COMPONENTS.md`
- `_harness/HARNESS_MATURITY.md`
- `_harness/IMPROVEMENT_PROTOCOL.md`
- `docs/decisions/*`

## Acceptance Criteria

- Agent workflow says to run `tool check` before optional external tool
  capability lookup.
- Story proof docs explain `story add/update --verify`, `story verify`, and
  `story verify-all`.
- Existing CLI capabilities for audit, component attribution, maturity, and
  proposals have `_harness/` docs.
- `_harness/TEST_MATRIX.md` includes the implemented `US-002` fallback row.
- Durable decision rows exist for current markdown decisions.
- The layout remains `_harness/` for agent runtime and `docs/product`,
  `docs/stories`, `docs/decisions` for product records.

## Design Notes

- Commands: `_harness/bin/harness-cli tool check`, `story verify`,
  `story verify-all`, `audit`, `propose`, `score-context`.
- Queries: `_harness/bin/harness-cli query matrix`, `query decisions`,
  `query backlog`.
- API: none.
- Tables: no schema shape change.
- Domain rules: keep current path contract; adapt experimental workflow docs to
  `_harness/` paths instead of restoring `docs/HARNESS.md` or `scripts/bin`.
- UI surfaces: agent-facing markdown and CLI output only.

## Validation

When updating durable proof status, use numeric booleans:
`_harness/bin/harness-cli story update --id US-003 --unit 1 --integration 1 --e2e 0 --platform 0`.

| Layer | Expected proof |
| --- | --- |
| Unit | `cargo test -p harness-cli` |
| Integration | CLI queries for matrix, decisions, tools, audit, and story verification command shape |
| E2E | not applicable |
| Platform | `bash -n install.sh`; `_harness/bin/harness-cli help` exposes `_harness/bin/harness-cli` |
| Release | not run |

## Harness Delta

Added workflow docs adapted from experimental, updated policy files, synchronized
fallback matrix, and seeded durable decision records.

## Evidence

- `cargo test -p harness-cli` passed with 27 tests.
- `bash -n install.sh` passed.
- `git diff --check` passed.
- `_harness/bin/harness-cli help` exposed `_harness/bin/harness-cli`.
- `_harness/bin/harness-cli tool check` completed.
- `_harness/bin/harness-cli query tools --summary` listed compiled commands,
  including `story verify`, `story verify-all`, `audit`, `propose`, and
  `score-context`.
- `_harness/bin/harness-cli query matrix` showed `US-001`, `US-002`, and
  `US-003`.
- `_harness/bin/harness-cli story verify US-003` passed.
