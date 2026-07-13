# US-005 Filter Query Friction

## Status

implemented

## Lane

normal

## Product Contract

`_harness/bin/harness-cli query friction` returns only traces whose
`harness_friction` value describes actual friction. Null, empty, whitespace-only,
and normalized `none` values are excluded.

## Relevant Product Docs

- `_harness/TRACE_SPEC.md`
- `_harness/TOOL_REGISTRY.md`

## Acceptance Criteria

- Traces with null, empty, or whitespace-only `harness_friction` are excluded.
- Traces whose trimmed, case-insensitive `harness_friction` value is `none` are
  excluded.
- Traces with actual friction remain ordered newest first and retain linked
  intake context.

## Design Notes

- Commands: `_harness/bin/harness-cli query friction`.
- Queries: filter `trace.harness_friction` in the SQLite repository.
- API: none.
- Tables: no schema change.
- Domain rules: use the same normalization already applied by repeated-friction
  proposal analysis.
- UI surfaces: agent-facing CLI table output only.

## Validation

When updating durable proof status, use numeric booleans:
`_harness/bin/harness-cli story update --id US-005 --unit 1 --integration 1 --e2e 0 --platform 0`.

| Layer       | Expected proof |
| ----------- | -------------- |
| Unit        | `cargo test -p harness-cli` |
| Integration | Rebuild the local CLI and verify `query friction` against `harness.db` |
| E2E         | not applicable |
| Platform    | not applicable |
| Release     | not run |

## Harness Delta

Correct failure-attribution output and add regression coverage for values that
explicitly mean no friction.

## Evidence

- `cargo test -p harness-cli` passed with 27 tests.
- `cargo clippy -p harness-cli -- -D warnings` passed.
- `./install-harness-cli.sh` rebuilt and installed the release binary.
- On the current `harness.db`, `query friction` returned 14 actual-friction
  rows instead of 22 non-null rows and returned no `none` values.
- `git diff --check` passed.
