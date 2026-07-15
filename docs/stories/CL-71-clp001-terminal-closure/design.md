# CL-71 Design and Decisions

## Domain Rules

- A historical audit disposition names an exact finding key and entity and
  stores status, rationale, provenance, approval source, actor, creation time,
  and optional expiry/revocation.
- Accepted findings remain visible separately in human and JSON audit output.
- Strict audit passes only when unresolved findings and unknown coverage are
  both absent.
- Expired or revoked dispositions become unresolved again.
- Health failures, unknown coverage, destructive recovery, and weakened
  validation cannot be accepted through a disposition.
- Historical records and retained recovery databases are immutable evidence;
  canonical current records are created only through CLI write paths.

These audit semantics are proposed by CLP-001 and remain implementation-blocked
until the required `architecture-direction` and `risk-policy` human approvals
are recorded.

## Application and CLI Contract

- `tests/release_qualification.sh` materializes `git diff --binary HEAD`, applies
  it only when non-empty, copies untracked files, and creates a candidate commit
  only when the clone has staged changes.
- The CLI provides command-first add/list/revoke operations for audit
  dispositions.
- Story reconciliation uses story-linked proof and real current task/intake
  traces; it never claims unobserved historical execution.
- Backlog recovery reads retained databases through read-only interfaces, then
  creates and closes a canonical successor through `harness-cli backlog`.

## Data and Migration Ownership

- A canonical source migration owns durable audit dispositions and its indexes
  or constraints.
- The packaged CLI and operational database move through the existing backup,
  migration, doctor, rollback, and distribution qualification contracts.
- No direct operational SQL write is permitted.

## Observability

- Proof runs retain command, exit status, output/artifact provenance, branch,
  commit, and dirty-state freshness.
- Audit output separates unresolved, accepted, expired, and revoked findings.
- Task traces and capsules identify actual work performed and discovered
  friction.

## Alternatives Rejected

1. Fabricating retrospective traces or task roots: contradicts historical
   evidence.
2. Suppressing findings in audit queries: hides debt and weakens validation.
3. Re-inserting legacy backlog id `4` through SQL: loses command provenance and
   violates the operational write boundary.

