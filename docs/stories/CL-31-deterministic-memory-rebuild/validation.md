# CL-31 Validation

Required proof: clean temp rebuild, duplicate conflict, invalid reference,
legacy/v1 parity, idempotent logical output, doctor validation, original DB
hash unchanged, and crash-safe cleanup/switch preparation.

Current evidence: `memory rebuild --dry-run` validates all artifacts, creates
the canonical schema in a sibling temporary database, projects checked artifact
metadata into `artifact_index`, runs doctor successfully, then removes the
temporary DB. The packaged command reports `projected_records: 28`.

CL-31 acceptance is complete: projection includes story/decision semantic
records and legacy Evidence text, duplicate/reference conflicts fail before
write, repeated dry-runs have an identical logical digest, and the explicit
apply path performs backup plus atomic replacement only after health checks.

The current projection now includes canonical `story` and `decision` rows.
Two independent packaged dry-runs produced the same logical digest:
`d4862f05422e330e81a3c80e57656079bfcd313711db2e130adbc060978f1f60`.
Artifact validation supplies duplicate/reference conflict reporting before a
temporary database is created. Legacy proof evidence is copied into
`story.evidence`; boolean proof is intentionally not inferred from prose.

`memory rebuild --dry-run --output <new-relative-path>` may retain a validated
new DB for inspection, but rejects absolute/traversal paths and any existing
file. It is not a switch command and cannot overwrite `harness.db`.

`memory rebuild --apply` is the separate explicit switch path. By default it
accepts only a `HEALTHY` or missing active DB. A reviewed foreign recovery must
also pass `--recover-foreign`; the command accepts only `DB_UNHEALTHY` or
`DB_AHEAD_OF_SOURCE`, requires the rebuilt candidate to pass doctor, then
checkpoints and backs up the quarantine DB before an atomic replacement.
`DB_UNREADABLE` remains rejected. The fresh-target apply fixture passed doctor
strict health.

The reviewed 2026-07-15 recovery used `--apply --recover-foreign` against the
quarantined version-`008` input. The command retained
`harness.db.backups/rebuild-607656.db`, atomically installed a canonical
version-`009` DB, and post-switch doctor, integrity and foreign-key checks
passed. No foreign story or task link was projected.
