# CL-72 Design and Decisions

## Approval State

The user approved all CLP-001-R1 human gates on 2026-07-15. `TASK-000025`
stores five scoped approval records under the canonical
`architecture-direction`, `source-hierarchy`, and `risk-policy` policy gates.
The accepted contracts are recorded in decisions 0022 through 0024:

1. Normalize typed intake values across space, underscore, and hyphen forms.
2. Default `--behavior-bearing` to conservative `auto`, derived only from typed
   intake, explicit flags, and a linked story; never inspect summary prose.
3. Use packet `overview.md` as identity and hash allowed components in sorted
   repository-relative order.
4. Keep v1 capsules readable while a richer compatible schema projects only
   observed critical task summaries and links.
5. Make `task finish` select exactly one qualifying rooted trace when `--trace`
   is omitted, retaining explicit trace selection for recovery/idempotency.
6. Treat semantic parity as proven only when a named, current check includes
   recursive discovery, projection, actual candidate schema, counts, IDs,
   paths, statuses, checksums, links, and required proof summaries.

## Invariants

- No summary-language inference.
- No symlink escape, traversal, duplicate identity, case collision, or unsafe
  file type enters the canonical artifact set.
- No historical task, trace, proof, approval, or disposition is synthesized or
  rewritten.
- `memory rebuild --dry-run` is non-destructive; `--apply` is backup-first,
  validated, and atomic.
- Strict audit cannot report required semantic coverage as covered without a
  named current `pass` result.

## Decision Records Required

After human approval, capture the accepted packet hierarchy/checksum contract,
capsule projection boundary, audit coverage semantics, auto-classification
table, and trace-selection rules in canonical decision artifacts before the
corresponding high-risk implementation.
