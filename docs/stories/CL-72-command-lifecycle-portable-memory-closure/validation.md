# CL-72 Validation and Evidence

## CGR-00 Baseline

- Bootstrap task: `TASK-000024`; tiny, non-behavior-bearing, owner `codex`,
  session `codex-clp001-r1-20260715`.
- Required context: `<changed-files>`; acknowledged after inspecting the clean
  starting worktree.
- Starting `HEAD`: `f1ab7f157a726faad0f638380af5f26298be8603` on
  `feature-rework`.
- Before this packet, canonical filesystem counts were 14 story-packet
  directories, 13 top-level legacy story files, 20 decision files, and 14
  nested task capsules.
- Current memory discovery checked 31 direct artifacts: 12 stories and 19
  decisions; it checked and projected zero capsules and skipped all packet
  directories.
- `memory rebuild --dry-run --json` reported `artifacts_checked=31`,
  `projected_records=31`, `temp_schema_version=7`, and logical digest
  `958dec14e14bf88aff29ea5ba398012222fd7ea7c00ebbefc2c0db9a09f79e6d`.
- Operational `artifact_index`: 12 story rows, 19 decision rows, zero capsule
  rows.
- Source and operational migrations both span `001..012`; every operational
  checksum matches its source file. `doctor --strict --json` was `HEALTHY` at
  commit `f1ab7f1` on Linux `x86_64`.
- Retained pre-012 backup:
  `harness.db.backups/harness.db.1784104246235156032.v11.main.a5af5ab9060c.bak`,
  SHA-256
  `2f062c0cecad5666bb436dd49389f270f7626d701beaca45afa76579c51507cc`.
- Operational DB SHA-256 after registering CL-72 and starting the bootstrap:
  `62860cf3fc8c15dbe1debc17925aa36ed7e23c933a829b5e83701360f861be70`
  at the time sampled. This is observational evidence, not a portable DB
  identity and may change as lifecycle records are added.

## Frozen Contract Suite

`bash tests/clp001_r1_contracts.sh baseline` runs each desired contract probe
and succeeds only when every listed gap is still reproduced. After fixes,
`bash tests/clp001_r1_contracts.sh all` must pass.

The suite covers the exact minimal start command, start/status JSON schemas,
JSON usage envelopes, minimal finish help, packet and nested capsule discovery,
corrupted capsule rejection, duplicate packet identity, symlink escape,
rebuild projection/schema reporting, and named audit coverage.

## CGR-10 Lifecycle Contract Evidence

- Task: `TASK-000025`; normal, behavior-bearing, owner `codex`, session
  `codex-clp001-r1-cgr10`; context `2/2` acknowledged.
- Human approval: the user replied `phê duyệt tất cả` on 2026-07-15. Five
  scoped durable records map the lifecycle, source hierarchy, audit policy,
  capsule projection and retained-DB safety directions to canonical policy
  gates.
- Decision 0022 records typed `behavior-bearing=auto`, explicit owner/session
  auto-assignment and exactly-one rooted trace selection. Decisions 0023 and
  0024 record the approved CGR-20 and CGR-30 directions.
- Source and packaged CLI `0.1.10` both pass ten command-contract probes:
  minimal start, complete start/status schemas, domain/parse/preflight/policy
  JSON envelopes, typed auto classification across space/underscore/hyphen
  input aliases, executable finish without `--trace`, and optional-trace help.
- The unit suite has 78 tests, including zero/one/multiple qualifying trace
  selection. Source Phase 4 and packaged Phase 4 preserve structured human/JSON
  errors, filesystem/DB non-mutation, ownership conflicts and capsule recovery.
- Installer state safety and workflow parity pass with the packaged binary.
- Capsule:
  `docs/tasks/2026/07/TASK-000025-completed-lifecycle-start-status-finish-contracts.md`.
- Detailed rooted trace: `#33`; achieved tier `detailed` against required
  normal-lane tier `standard`.
- Structured friction
  `6a50de1e05e6cf7b29a77f8ff564e8ec58a6552c6679f1780d271076020013e5`
  is `validated`; no material friction remains unresolved.
- The terminal CGR-40 evidence update appends the final proof hash and confirms
  the durable closure result without rewriting this task history.

## CGR-20 Portable Semantic Rebuild Evidence

- Task: `TASK-000026`; high-risk, behavior-bearing, owner `codex`, session
  `codex-clp001-r1-cgr20`; context `5/5` acknowledged and scoped
  `architecture-direction`, `source-hierarchy`, and `risk-policy` approvals
  recorded from the user's explicit approval of all gates.
- Source and packaged CLI `0.1.11` discover 64 current artifacts before this
  task capsule: 27 stories (15 packet identities plus 12 legacy single files),
  22 decisions, and 15 nested capsules. Packet components are sorted and hashed
  into one aggregate story checksum.
- Migration `013-portable-memory-projection.sql` adds the rebuildable
  `portable_task_summary` projection. The retained operational DB was migrated
  through the supported backup-first path from schema `12` to `13`; backup
  `harness.db.1784113667748362911.v12.main.1973983375ec.bak` remains retained.
- Existing v1 capsules remain readable and explicitly mark story, trace, and
  proof links unknown. Newly rendered v2 capsules contain observed story IDs,
  rooted trace IDs, and structured proof summaries; no historical fields are
  inferred.
- Eleven source and packaged memory contracts pass: recursive packet/capsule
  discovery, corrupt capsule, duplicate ID, symlink escape, case collision,
  unsafe file type, actual-schema projection, repeatable/non-destructive
  dry-run, aggregate component checksum, and backup-first apply that preserves
  operational task rows.
- Two independent dry-run candidates report schema `13`, 64 projected records,
  15 portable task summaries, zero mismatches, and identical logical digest
  `5b53085e5764e7a1a2663ec93f1f04421aa8b90176c66958fbe2b97e3b99e253`.
- Candidate `doctor` is `HEALTHY`. Candidate audit is deliberately not called
  clear while the old inferred semantic coverage remains unknown; CGR-30 owns
  the named current coverage record and strict-audit transition.
- Initial structured proof passed with stdout hash
  `392c1db4147ffcec6937ffd3af032ef8a2efd514c98b13bfa6a70bf02556134a`
  and stderr hash
  `ca4ba0b43583198a49c85e875cda1a00e6f472e8bb2469c73cad2161580f66f6`.
- Detailed rooted trace: `#34`; achieved the required high-risk tier `3/3`.
  v2 capsule:
  `docs/tasks/2026/07/TASK-000026-completed-recursive-safe-artifact-discovery-aggregate.md`.
- Retained rebuild apply started from a copy of the healthy operational DB,
  created backup `harness.db.backups/rebuild-1784114017049537313.db`, passed
  candidate doctor and parity, then atomically refreshed 65 artifact rows:
  27 stories, 22 decisions and 16 capsules, including one v2 portable summary.
  Counts for tasks `26`, traces `34`, proof runs `125`, approvals `16`, audit
  dispositions `2`, and friction records `6` match the pre-apply backup.
- Structured friction
  `14001ec69e43a01be3b926022e7da162fa9f9f0af442e8d99a5b59fc5ae0c723`
  records that legacy backup retention briefly pruned three Git-tracked v8
  recovery files during migration. The exact HEAD blobs were restored and the
  policy now excludes tracked backups and sidecars from pruning; both ordinary
  five-backup retention and protected-backup regression tests pass.

## CGR-30 Named Audit and Evidence Reconciliation

- Task: `TASK-000027`; high-risk, behavior-bearing, owner `codex`, session
  `codex-clp001-r1-cgr30`; context `4/4` acknowledged and the approved
  `risk-policy` boundary recorded before implementation.
- Source CLI `0.1.12` replaces the broad CL-70 release-observation inference
  with five named versioned checks: `markdown-db-field-parity`,
  `path-scoped-proof-freshness`, `generated-matrix-cli-payload-parity`,
  `semantic-memory-parity`, and `operational-telemetry`.
- Every check exposes state, proof/command and output provenance,
  HEAD/branch/dirty/output freshness, scope, measured counts, and exact
  remediation. Strict audit treats every required non-pass state as unknown
  coverage.
- Seven audit contracts pass: named registry presence, absent=`unknown`, command
  failure=`fail`, complete current parity=`pass`, incomplete current output=
  `fail`, omitted indexed artifact=`fail`, and stale proof=`unknown` with
  `dirty=false`. A broad CL-70 release observation cannot promote these checks.
- Canonical current story
  `docs/stories/CL-01-canonical-architecture-decision-package.md` records the
  accepted ADR package without claiming a historical CL-01 task. It is
  registered and verified through supported commands.
- `TASK-000027` links CL-72 as primary and 23 retained CLP-001 stories as
  secondary current-reconciliation scope. This is current evidence and does
  not alter or fabricate prior tasks, traces, proofs, approvals or
  dispositions.
- Structured friction
  `cdc29d2b1ebb6299d06dfc07af084ecfca34816abcafcf21af0310ecb0f6536c`
  records generic `Overview` titles introduced by retained packet projection.
  Packet titles now derive deterministically from canonical directory identity,
  and the rebuild contract asserts zero generic CL packet titles.
- The Rust suite has 79 tests; the complete source CLP-001-R1 contract suite
  passes command, memory and audit modes.
- Initial structured proof passed with stdout hash
  `148da0a53cf70237c18d3edc9345331cbf0ddb99eb713f237c74524b7df21465`
  and stderr hash
  `dbbb744607a98449f7669d665d0db7f7dfd7a8c394a97684bf01baca9b2a9b36`.
- Detailed rooted trace: `#35`; achieved the required high-risk tier `3/3`.
  v2 capsule:
  `docs/tasks/2026/07/TASK-000027-completed-named-current-audit-coverage-with.md`.
  It contains the observed 24 story links, trace ID and proof summary.
- Retained projection refresh created backup
  `harness.db.backups/rebuild-1784115173199390349.db` and projected 67
  artifacts: 28 stories, 22 decisions, and 17 capsules. Candidate doctor and
  semantic parity passed with zero mismatches; generic `Overview` titles are
  zero.
- Initial named proof runs `128`, `129`, `133`, `131`, and `132` established
  the five-check result contract before final documentation reconciliation.
  The repeated terminal runs are retained in audit/proof evidence; semantic
  counts are stories `28`, decisions `22`, capsules `17`, projected `67`,
  schema `13`.
- Strict audit is `AUDIT_CLEAR`: all five required checks are `pass`, unknown
  coverage is empty, all unverified story/decision and unresolved friction
  lists are empty, accepted historical dispositions remain visible, and H5
  remains achieved.
- Structured friction
  `16c411a0d5c90e3dcddd4e82cba267e3b945253991787661601335edbb875ac7`
  records the first generated-payload proof's invalid source-manifest path.
  Latest proof `133` uses installer safety, observed packaged binary bytes and
  compiled command counts and is a current `pass`; the failed attempt remains
  visible rather than being rewritten.

## Human Approval Questions

The user answered `phê duyệt tất cả` on 2026-07-15. The lifecycle, packet,
capsule, audit-policy, and retained backup-first apply questions above are
therefore approved. Scoped durable approval records remain attached to the
tasks that own each high-risk action; `TASK-000028` records the terminal
`risk-policy` approval separately.

## CGR-40 Terminal Requalification

- Terminal task: `TASK-000028`; high-risk, non-product, owner `codex`, session
  `codex-clp001-r1-cgr40`; started from clean committed HEAD `6e464a5` after
  CGR-10 through CGR-30 were committed together.
- Required context is acknowledged `4/4`; the scoped `risk-policy` approval
  cites the user's explicit approval of all gates.
- The terminal proof ladder covers format, workspace Clippy/tests, shell
  syntax, installer safety, release qualification in `state`, `distribution`,
  and `all` modes, workflow parity, source and packaged CLP-001-R1 contract
  suites, memory check/rebuild/capsule validation, strict doctor/audit, and
  `git diff --check`.
- Five named clean-HEAD proof layers retain their own proof IDs, command and
  output hashes, measured counts, HEAD/branch/dirty/output freshness, and
  remediation in operational evidence. They are not copied into this tracked
  file after the final proof because doing so would change HEAD and make the
  evidence stale.
- The current portable projection uses canonical schema `13`. The terminal
  refresh includes every discovered story, decision, and nested capsule with
  zero parity mismatches; legacy capsule unknowns remain explicit.
- The CLI-rendered v2 capsule for `TASK-000028` contains only observed story,
  trace, and proof summaries. `task finish` owns the final atomic status and
  lease release; task status, doctor, audit, memory, capsule, and Git checks are
  read-only afterward.
- Rollback remains coherent: revert the lifecycle/parser/audit/package/docs
  commits together; for retained state, restore the named pre-change backup
  with the matching prior binary through supported recovery. Preserve proofs,
  traces, approvals, dispositions, friction, and capsules.
- Compatibility remains N+2: retain v1 capsule reading, legacy commands and
  parsers, and `schema_version`; no platform expansion or early removal is
  authorized.

## Proof Matrix

| Layer | Expected proof | Current result |
| --- | --- | --- |
| Bootstrap | `bash tests/clp001_r1_contracts.sh baseline` | 15 intended gaps reproduced by `TASK-000024` |
| Unit/static | workspace format, Clippy, and tests | pass; 79 Rust tests |
| Integration | source/package contract and semantic fixtures | pass; 28 probes on each CLI surface |
| Audit | five named current semantic/freshness checks | pass with zero unknown required coverage |
| Release | installer safety and full release qualification | pass on Linux `x86_64` |
| Terminal | complete clean-HEAD ladder and post-finish checks | owned by `TASK-000028` durable evidence |
