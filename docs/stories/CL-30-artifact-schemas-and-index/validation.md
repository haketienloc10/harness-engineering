# CL-30 Validation

## Proof Strategy

Use temp repositories/databases. The current repository-local DB remains a
foreign/ahead recovery input and is not a CL-30 test target.

## Test Plan

| Layer | Cases |
| --- | --- |
| Unit | Legacy and v1 parse; checksum; required fields; path normalization |
| Integration | Duplicate IDs/paths; missing references; index projection schema |
| Black-box | `story check`, `decision check`, `memory check --dry-run` success/fail JSON |
| Platform | Packaged binary command manifest and installer payload |

## Acceptance Evidence

- `artifact_index` migration `007` is checksummed in the canonical main
  manifest and is exercised by the existing migration/doctor fixtures.
- `memory check --dry-run --json` validates every current top-level legacy
  story and decision without writing documents or the local database.
- Unit proof includes a duplicate v1 story-ID fixture and verifies input files
  remain byte-identical after checking.
- Passed: `cargo test -p harness-cli` (55), `cargo clippy -p harness-cli --
  -D warnings`, `bash tests/installer_state_safety.sh`, and packaged
  `memory check --dry-run --json`.

V1 checks now require type-specific fields, valid story lane, safe existing
`product_docs` references, and a safe semantic ID. A malformed lane/missing
reference fixture is covered in addition to duplicate IDs.
