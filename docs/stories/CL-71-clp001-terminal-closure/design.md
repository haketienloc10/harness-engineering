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

These audit semantics were approved by the user and recorded under
`TASK-000020` as the required `architecture-direction` and `risk-policy`
approvals before implementation or operational migration.

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

- Canonical migration `012-audit-disposition.sql` owns durable audit
  dispositions, the approval-task foreign key, allowed finding keys, accepted
  uniqueness, revocation invariants and indexes.
- The packaged CLI and operational database move through the existing backup,
  migration, doctor, rollback, and distribution qualification contracts.
- No direct operational SQL write is permitted.

## Observability

- Proof runs retain command, exit status, output/artifact provenance, branch,
  commit, and dirty-state freshness.
- Audit output separates unresolved, accepted, expired, and revoked findings.
- Task traces and capsules identify actual work performed and discovered
  friction.
- `audit disposition list` shows accepted, expired and revoked durable rows;
  `audit` separately shows only currently effective accepted findings.

## Alternatives Rejected

1. Fabricating retrospective traces or task roots: contradicts historical
   evidence.
2. Suppressing findings in audit queries: hides debt and weakens validation.
3. Re-inserting legacy backlog id `4` through SQL: loses command provenance and
   violates the operational write boundary.
