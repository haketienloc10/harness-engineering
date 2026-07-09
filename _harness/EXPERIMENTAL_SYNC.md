# Experimental Synchronization Baseline

Last synchronized: 2026-07-09

Source repository:
`/home/locdt/Notes/VSCode/harness-experimental`

Source commit:
`14e6f102a4a645562d046f7c693c61401261cac6`

Destination layout decision:
`docs/decisions/0010-experimental-sync-with-stable-layout.md`

## Path Mapping

| Experimental source | Destination |
| --- | --- |
| `docs/<runtime-policy>.md` | `_harness/<runtime-policy>.md` |
| `docs/templates/` | `_harness/templates/` |
| `scripts/bin/harness-cli` | `_harness/bin/harness-cli` |
| `scripts/schema/` | `_harness/scripts/schema/` |
| `crates/harness-cli/` | `crates/harness-cli/` |
| `crates/harness-symphony/` | `crates/harness-symphony/` |

## Exclusions

- `target/`
- `UNCOMMITTED_CHANGES_REPORT.md`
- Source operational `.harness/` state and changesets
- Historical phase notes and source-only story history
- Experimental repository branding and release URLs

Future updates should compare the experimental repository from the source
commit above and update this baseline in the same destination commit.
