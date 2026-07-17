# Proposal: Closed-Loop Agent-First Harness

Date: 2026-07-13

Status: Proposed

Review scope: `AGENTS.md` → Harness policy/docs → Rust CLI → `harness.db` →
task/friction/improvement lifecycle.

Review baseline:

- Repository: `harness-engineering`
- Branch: `main`
- Commit: `ae580d7446b6d37a578fcf386f98f8612fe6cffe`
- CLI: `harness-cli 0.1.9`
- Local database snapshot before this proposal: 23 intakes, 3 stories, 4
  decisions, 3 backlog items, 25 traces.

## 1. Executive Decision

Keep the current three-layer model, but make each layer's authority explicit:

1. Version-controlled Markdown is portable semantic memory: product truth,
   story intent, decisions, compact completed-task capsules, and accepted
   Harness policy.
2. The CLI is the only normal write gateway for operational Harness state and
   the only gate allowed to declare a task complete.
3. `harness.db` is a local, rebuildable operational index and event store. It
   must not be treated as the only durable project memory because it is ignored
   by Git and currently survives branch switches without branch/schema checks.

Replace the current manually coordinated sequence with two primary commands:

```text
harness-cli task start ...
  -> doctor
  -> intake + lane
  -> context manifest
  -> optional story attachment

harness-cli task finish ...
  -> validation/proof checks
  -> docs/DB consistency checks
  -> trace quality gate
  -> friction disposition
  -> atomic task closure
  -> portable task capsule
```

Do not implement self-improvement automation before the state and completion
gates are trustworthy. Current `audit` and `propose` can report a perfect state
while docs, migrations, stories, and decisions disagree.

## 2. Review Method And Evidence

The review followed the repository intake contract and inspected:

- Entrypoint and policy: `AGENTS.md`, `_harness/HARNESS.md`,
  `_harness/FEATURE_INTAKE.md`, `_harness/CONTEXT_RULES.md`.
- Architecture and lifecycle references: `_harness/ARCHITECTURE.md`,
  `_harness/TRACE_SPEC.md`, `_harness/HARNESS_AUDIT.md`,
  `_harness/HARNESS_COMPONENTS.md`, `_harness/HARNESS_MATURITY.md`,
  `_harness/IMPROVEMENT_PROTOCOL.md`, `_harness/TOOL_REGISTRY.md`.
- Product records: all current story and decision Markdown files and templates.
- CLI contract: top-level and subcommand help, compiled command registry, Rust
  domain/application/infrastructure/interface code, and the installed binary.
- Durable state: schema, table/index inventory, foreign-key and integrity
  checks, current records, linkage, audit output, proposal output, and a fresh
  temporary database initialized from `main`.
- Distribution: `install.sh`, `install-harness-cli.sh`, `.gitignore`, binary
  placement, and the embedded target-repository `AGENTS.md` block.
- Validation: `cargo test -p harness-cli`, `cargo clippy -p harness-cli -- -D
  warnings`, `cargo fmt --all -- --check`, shell syntax checks, and
  `git diff --check`.

Key observed results:

- SQLite `PRAGMA integrity_check` returned `ok`; `foreign_key_check` returned no
  violations.
- Rust tests passed: 27/27.
- Clippy passed with warnings denied.
- Shell syntax and `git diff --check` passed before this proposal.
- `cargo fmt --all -- --check` failed on pre-existing formatting drift in
  `domain.rs` and `infrastructure.rs`; backlog item #2 already records it.
- `audit` returned `0/100` and `propose` returned no proposals, despite the
  consistency failures below.

## 3. Current Workflow: What Actually Happens

The intended path is coherent on paper:

```text
intent
  -> intake
  -> lane
  -> story when needed
  -> implementation or blocker
  -> validation
  -> trace
  -> friction fix or backlog
```

The actual runtime is a set of independent writes:

```text
Markdown edits ───────────────┐
                             ├─ no shared transaction or reconciliation gate
CLI intake/story/trace writes ┘

validation command
  -> human/agent interprets output
  -> agent separately sets proof booleans
  -> agent separately records trace
  -> trace scorer prints warnings
  -> task may still be reported complete
```

This is an open loop because no single record represents the task lifecycle,
no command owns the completion invariant, and the final trace can be inserted
even when its lane, proof, docs, friction, or schema state is incomplete.

## 4. Findings

### 4.1 P0 — Database Schema Can Belong To Another Branch

What happens:

- The current `main` source contains migrations `001` through `005`.
- The local `harness.db` reports applied versions `001` through `008`.
- It contains `changeset_applied`, `story_dependency`, and `story_hierarchy`,
  which came from the `SYMPHONY` branch.
- It also contains `US-004` and decision
  `0010-experimental-sync-with-stable-layout`, neither of which exists in the
  `main` working tree.
- `_harness/bin/harness-cli migrate` prints:

```text
Current schema version: 8
Already up to date.
```

Why it happens:

- `harness.db` is ignored and stored at repository root, so checking out another
  branch does not change or reset it.
- `migrate` applies files whose version is greater than the DB maximum, but does
  not reject a DB version newer than the largest migration available in the
  current checkout.
- No database metadata binds state to repository identity, migration lineage,
  branch/worktree, or CLI build.

Impact:

- A branch can read or mutate operational records created by incompatible code.
- `audit`, matrix, and proposals can use facts that do not belong to the current
  source tree.
- Migration numbers can collide across branches.
- “Already up to date” is a false safety claim.

Required action:

- Add `doctor` and fail closed when `db_max_version > source_max_version`, when
  migrations have gaps/duplicate versions, or when checksums differ.
- Store `repository_id`, `schema_lineage`, `created_by_cli_version`, and last
  validated Git branch/worktree/commit in database metadata.
- Define one migration-lineage rule: shared `harness.db` may only be migrated by
  migrations present on the canonical branch; branch-only experiments use an
  explicitly separate `HARNESS_DB`.
- Back up before migration and apply each migration transactionally.

### 4.2 P0 — Startup Does Not Upgrade An Existing Database

What happens:

- `AGENTS.md` says to run `init` only when `harness.db` is missing.
- The required startup sequence queries the matrix before any `migrate` step.
- `init` applies migrations for a new/unversioned database, but an existing
  versioned database returns immediately without applying pending migrations.
- A temporary version-1 simulation remained at version 1 after `init`.

Impact:

- Installing a new Harness payload over a target repository can leave its
  database behind the CLI/schema expected by the new payload.
- The first task after an upgrade may fail during an unrelated query.

Required action:

- Make `init` idempotently run `doctor` and pending migrations, or replace it
  with `ensure` that owns create + migrate + compatibility checks.
- Make `task start` call this automatically before any query.
- Installer output must explicitly report whether durable state is created,
  migrated, incompatible, or intentionally untouched.

### 4.3 P0 — `audit 0/100` Does Not Mean Consistent

Observed drift on the reviewed workspace:

| Surface | Version-controlled records | Local DB records |
| --- | --- | --- |
| Stories | `US-001`, `US-002`, `US-003`, `US-005` | `US-001`, `US-004`, `US-005` |
| Decisions | 7 Markdown ADRs | 4 rows; 4 docs missing from DB and 1 DB-only decision |
| Schema | migrations 1–5 | versions 1–8 |

Yet audit reports:

```text
Entropy score: 0/100 (lower is better)
```

The current audit only checks:

- unfinished stories with no linked trace;
- configured verify commands with no result;
- implemented backlog items with predicted impact but no outcome;
- unfinished stories older than 30 days;
- broken registered tools.

It does not check:

- source migration compatibility or checksums;
- Markdown ↔ DB existence/status/path/proof mismatch;
- missing or stale task closure;
- trace tier failures or missing intake links;
- proof freshness at the current commit;
- unresolved friction disposition;
- invalid artifact paths;
- compiled command manifest ↔ actual CLI mismatch;
- installer ↔ documented install-surface mismatch.

Required action:

- Split health from entropy. `doctor` answers “safe to operate?”; `audit`
  answers “how much drift/debt exists?”
- Replace “Perfect” with explicit measured coverage. Unknown/unmeasured must not
  score as zero defects.
- Return non-zero for P0 consistency failures and support `--json` for tests.

### 4.4 P0 — `query sql` Can Mutate Durable State

The command is named and documented as a query, but a test against a copied
database executed:

```sql
DELETE FROM backlog WHERE id=1
```

and changed the row count from 3 to 2 without an error or confirmation.

Impact:

- It bypasses application validation, lifecycle rules, and future audit hooks.
- A read-oriented agent command can silently destroy project memory.

Required action:

- Restrict `query sql` to SQLite read-only connections and read-only statements.
- Move writes to an explicit administrative command such as `admin sql --write`
  with backup, confirmation/force semantics, traceability, and clear warnings.
- Do not expose administrative SQL in the normal compiled capability manifest.

### 4.5 P0 — Task Completion Is Not A Gate

What happens:

- There is no task row/status; intake and trace are loosely related events.
- `trace --outcome completed` can be inserted without an intake.
- If intake is omitted, trace scoring has no lane requirement and exits zero.
- `trace` prints its score but does not fail when below the required tier.
- `score-context` is post-hoc and does not gate completion. Trace #24 met only
  2/4 compiled must-read rules but its task remained completed.
- `outcome` is optional at the CLI/schema boundary even though the trace spec
  says it is required.

Impact:

- An agent can bypass normal/high-risk trace requirements simply by omitting
  `--intake`.
- “Done” is a prose claim, not an enforced state transition.

Required action:

- Treat intake as the task root with `open`, `blocked`, `completed`, `failed`,
  and `abandoned` lifecycle states.
- Require one final trace for task closure; allow additional attempt traces.
- Make `task finish` validate lane, story, proof, trace tier, context, docs/DB
  consistency, and friction disposition before committing closure atomically.
- Require an explicit reason and durable follow-up for every skipped gate.

### 4.6 P1 — Ignored SQLite Is Not Portable Project Memory

What happens:

- `harness.db` is correctly ignored to avoid merge and machine-state problems.
- Intakes, traces, interventions, and backlog outcomes that live only in the DB
  disappear for a fresh clone, a new machine, CI, or another agent workspace.
- Current Markdown stories and decisions preserve some semantic history, but
  tiny/maintenance task outcomes and friction are not promoted automatically.

Impact:

- “Agents understand the project over time” works only in one long-lived local
  checkout.
- Audit and proposal quality depends on accidental database continuity.

Required action:

- Generate one compact, version-controlled task capsule on successful closure,
  for example `docs/tasks/YYYY/MM/TASK-<id>-<slug>.md`.
- A capsule contains only portable, reviewable knowledge: outcome, lane, linked
  story/decisions, start/end commit, changed surfaces, validation results,
  friction disposition, backlog links, and concise lessons.
- Exclude raw prompts, raw logs, secrets, and machine-specific noise.
- Add `memory rebuild` to reconstruct critical DB state from version-controlled
  stories, decisions, and task capsules; verify round-trip parity in CI.
- Update the durable-layer ADR: SQLite is the local operational index; Git
  records are the portable semantic memory.

### 4.7 P1 — Brownfield Import Produces Incorrect Semantics

A fresh temporary database initialized from `main` and then populated with
`import brownfield` showed:

- 4 stories and 7 decisions imported;
- every story assigned `risk_lane=high_risk`, including normal stories;
- `contract_doc` populated with the matrix's human-readable contract title,
  not a document path;
- story `verify_command` not imported from story packets;
- no traces imported;
- audit still reported `0/100` and `propose` returned nothing.

Additional risks:

- Matrix import can overwrite DB proof/status with a stale fallback table.
- The multi-surface import is not wrapped in one explicit transaction.
- `decision add` is insert-only although docs say it can “add or refresh”.
- There is no dry run, conflict report, source checksum, or import provenance
  beyond a free-text note.

Required action:

- Make import a one-time/rebuild workflow, never a silent two-way sync.
- Add `--dry-run`, conflict output, source checksums, and one transaction.
- Parse story packets as the semantic source; use fallback matrix only when no
  story packet exists.
- Preserve lane, document path, verify command, status, and evidence separately.
- Make sync direction explicit: `story index-from-docs`, `story export`, or
  `memory rebuild`, not ambiguous import/upsert behavior.

### 4.8 P1 — Proof Is Boolean, Manual, And Not Freshness-Bound

What happens:

- `story verify` stores only command, timestamp, and pass/fail.
- Proof booleans are updated by a separate command and can be set independently
  of the verification result.
- A prior pass remains valid after source, command, dependency, or environment
  changes.
- Audit treats any non-null pass/fail history as verified enough; it does not
  compare the proof with current `HEAD`.

Required action:

- Add a `proof_run` record with story, layer, normalized command, exit code,
  started/finished time, `HEAD`, dirty-worktree fingerprint, CLI version, and
  optional artifact path/hash.
- Derive matrix booleans/read models from successful proof runs instead of
  accepting independent claims.
- Mark proof stale when relevant files, command configuration, or commit scope
  changes.
- Define explicit `not_applicable`, `not_run`, `pass`, `fail`, and `stale`
  states; a boolean cannot distinguish them.

### 4.9 P1 — Friction Does Not Reliably Become Learning

What happens:

- Friction is a free-text field on trace.
- `propose` groups only fully normalized, effectively identical strings and
  requires at least two occurrences.
- Similar issues phrased differently do not aggregate.
- Backlog `discovered_while` is free text, not a trace/task foreign key.
- Backlog close permits missing actual outcome; `implemented_at` is set even for
  rejected items.
- There is no `implemented_pending_observation` state for improvements whose
  impact needs future tasks to measure.

Current evidence demonstrates the gap: 14 historical friction rows and one
intervention exist, but `propose` generated no proposal.

Required action:

- Store structured friction: component, category, severity, stable fingerprint,
  occurrence count, task/trace links, and disposition.
- Suggested categories: `policy-gap`, `docs-drift`, `schema-drift`,
  `validation-gap`, `tool-missing`, `repeated-manual-step`, `environment`, and
  `false-positive`.
- Every task closure must choose `fixed-now`, `backlog`, `accepted-risk`, or
  `not-friction` for each item.
- Link accepted backlog items to implementation stories and proof.
- Use lifecycle:

```text
proposed
  -> accepted
  -> in_progress
  -> implemented_pending_observation
  -> validated | ineffective | reverted
```

- Compare predicted impact against an explicit baseline and observation window,
  such as the next five related tasks or 30 days.

### 4.10 P1 — Source Hierarchy Mixes Normative Truth And Operational Proof

The current single hierarchy places matrix proof above decisions:

```text
user -> product docs -> stories -> matrix -> decisions -> code/tests
```

That conflates different questions. A matrix can say whether proof exists; it
cannot override an accepted architecture or product decision.

Required action:

Use concern-specific hierarchies:

```text
Behavior/contract truth:
current user intent -> product docs -> story acceptance criteria
-> accepted decisions -> code/tests

Execution/proof truth:
current proof runs -> task capsule/trace -> derived matrix

Harness policy truth:
AGENTS.md -> HARNESS.md/focused policy -> accepted Harness decisions

Operational index:
harness.db derives/indexes the above; it does not override them
```

Conflicts between concerns must be reported, not resolved through a single
global ordering.

### 4.11 P1 — Documentation And Distribution Contracts Drift

Observed examples:

- `README.md` lists `.gitignore` as installed surface, but `install.sh` does not
  include `.gitignore` in `INSTALL_ITEMS`; a target can accidentally track
  `harness.db`.
- The target `AGENTS.md` workflow is duplicated as a large literal block inside
  `install.sh`, creating two manually synchronized policy sources.
- `TOOL_REGISTRY.md` lists `tool check`, but the compiled outbound registry
  returned by `query tools` omits it.
- Decision 0004 says `sqlite3` is required, while the Rust CLI uses bundled
  SQLite; backlog #3 already records this stale statement.
- `_harness/HARNESS_COMPONENTS.md` calls project memory “Covered”, although
  operational memory is not portable and current docs/DB drift is invisible.
- H4 is marked achieved and H5 partial, but fresh import has no verify commands,
  proof freshness is not enforced, and measured improvement outcomes are absent.
- The tracked CLI binary is Linux x86-64; the Windows wrapper expects an `.exe`
  that is not part of the installed tracked payload. Cross-platform posture is
  documented as future work but not surfaced as a runtime compatibility check.

Required action:

- Generate the installed `AGENTS.md` shared core from one template/source and
  test byte-level parity for shared sections.
- Generate the compiled command manifest from the CLI definition or test that
  every public command is represented exactly once.
- Add installer contract tests for installed paths, ignore rules, upgrade
  behavior, stale-file handling, and platform compatibility messages.
- Recalibrate maturity claims to evidence, not feature presence.

### 4.12 P2 — Data Model And Queryability Need Hardening

Small issues become important over many tasks:

- `intake.story_id` and `intervention.story_id` are not foreign keys.
- JSON-like list fields are `TEXT` without `json_valid` constraints.
- Comma-separated CLI input cannot safely represent values containing commas.
- Story status transitions are not enforced.
- Decision and backlog lifecycle transitions are weakly validated.
- `query intakes` and `query traces` return only 20 recent rows and lack normal
  filters/JSON output, reducing historical retrieval quality.
- Trace timestamps omit repository/branch/commit identity and environment
  provenance.
- No retention/redaction policy exists for operational traces.
- Multiple agents have no task ownership/lease convention; SQLite serializes
  writes but does not prevent two agents from completing the same logical task.

Required action:

- Add missing keys/indexes/checks and structured JSON arguments.
- Add stable `--json` output and filters by task, story, lane, outcome, date,
  component, and friction fingerprint.
- Record repository, branch/worktree, start/end commit, and agent/CLI version.
- Define trace retention, redaction, and capsule promotion rules.
- Add optional task owner/session identity and detect conflicting open work.

## 5. Target Closed Loop

### 5.1 Start

```bash
_harness/bin/harness-cli task start \
  --type harness-improvement \
  --summary "Review and close the Harness task loop" \
  --flags "public-contract,weak-proof,multi-domain"
```

The command must:

1. Locate repository root independently of the caller's current directory.
2. Run `doctor`: CLI/platform, DB integrity, schema lineage/version/checksum,
   Git identity, required paths, and pending migration checks.
3. Create or resume exactly one open task.
4. Compute/recommend lane from structured flags; record any override and reason.
5. Run `tool check` and report only relevant capability posture.
6. Return a phase/lane-specific context manifest with must/should/skip paths.
7. Require a story only when the task is behavior-bearing; record the reason
   when no story is needed.

### 5.2 Work And Proof

```bash
_harness/bin/harness-cli task status --id TASK-...
_harness/bin/harness-cli proof run --story US-... --layer unit
```

- The task owns links to stories, decisions, backlog items, proof runs, and
  attempts.
- Proof commands execute from a stable repository root and record commit/env
  provenance.
- Docs and DB are checked after producer writes, never read concurrently with
  dependent writes.
- High-risk direction changes remain human approval gates.

### 5.3 Finish

```bash
_harness/bin/harness-cli task finish \
  --id TASK-... \
  --outcome completed \
  --friction none
```

Preconditions by lane:

| Gate | Tiny | Normal | High-risk |
| --- | --- | --- | --- |
| Intake and lane complete | required | required | required |
| Story | when behavior-bearing | required when behavior-bearing | required packet |
| Relevant docs current | required | required | required |
| Proof | quick | story plan | full validation plan |
| Decision | when durable rule changes | when durable rule changes | required for meaningful high-risk change |
| Trace tier | Minimal/Standard if Harness changed | Standard | Detailed |
| Friction disposition | required | required | required |
| Docs/DB consistency | required | required | required |
| Portable capsule | required | required | required |

The closure transaction writes the final trace, task outcome, links, friction
disposition, and capsule metadata together. File generation is staged first;
the DB transaction commits only after the capsule can be written safely.

### 5.4 Next Task Retrieval

At the next `task start`, agents receive a compact context summary:

- current product/story/decision state;
- open or blocked tasks and active owners;
- proof stale/failing status;
- recent task capsules relevant to the affected surface;
- recurring friction and accepted improvements awaiting measurement;
- exact context files to read, with an explicit stop condition.

This gives historical understanding without loading every raw trace.

## 6. Proposed Durable Model

Prefer append-only evidence plus derived read models.

### 6.1 New Or Revised Records

| Record | Purpose | Portable |
| --- | --- | --- |
| `harness_meta` | repository/schema lineage, CLI compatibility, migration checksums | no; rebuild metadata |
| `task` | lifecycle rooted at intake, lane, owner, branch/worktree, start/end commit | capsule projection |
| `task_attempt`/`trace` | detailed execution attempts and final trace | summarized |
| `proof_run` | command/layer/result/commit/environment/artifact hash | summarized |
| `friction` | structured issue, fingerprint, component, severity, disposition | summarized when material |
| `task_friction` | occurrence/link between task and friction | summarized |
| `backlog` | accepted improvement lifecycle and predicted/actual outcome | material items exported |
| `artifact_index` | Markdown ID/path/checksum/status index | rebuildable |
| `migration_history` | version, checksum, CLI version, applied time | local, checked against source |

### 6.2 Task Capsule

Suggested path:

```text
docs/tasks/2026/07/TASK-0023-closed-loop-harness-review.md
```

Minimum content:

```yaml
task_id: TASK-0023
date: 2026-07-13
lane: normal
outcome: completed
story_ids: []
decision_ids: []
start_commit: ae580d7...
end_commit: <commit or working-tree fingerprint>
validation:
  - command: cargo test -p harness-cli
    result: pass
friction:
  - fingerprint: schema-lineage-cross-branch
    disposition: proposal
backlog_ids: []
```

Then concise Markdown sections: outcome, changed surfaces, decisions/non-goals,
validation gaps, and reusable lesson. Raw logs remain outside Git.

## 7. CLI Contract Changes

### 7.1 Add

```text
doctor [--strict] [--json]
task start/status/finish/abandon
story check/sync
decision check/sync/update
proof run/query
friction add/resolve/query
memory export/rebuild/check
audit --strict --json
```

### 7.2 Change

- `init`: create if missing, otherwise compatibility-check and migrate.
- `migrate`: transactional, checksummed, backup-first, fail if DB is ahead or
  lineage differs.
- `trace`: normally internal to `task finish`; standalone use requires an
  explicit diagnostic mode and cannot close a task.
- `story update`: enforce status transitions and derive proof state from runs.
- `decision add`: either true idempotent upsert or separate `add`/`update`; docs
  must match behavior.
- `backlog close`: require status-specific outcome; rejected must not receive an
  implementation timestamp.
- `propose`: group structured fingerprints, deduplicate open backlog, and never
  auto-commit high-risk policy proposals.
- `query *`: stable JSON, pagination/filtering, and documented exit codes.

### 7.3 Restrict

- `query sql`: read-only.
- shell verification strings: keep only as an explicit trusted-repository
  capability; prefer structured executable/argument form where possible.
- direct proof booleans: remove or mark administrative/import-only.

## 8. Audit And Maturity Redesign

### 8.1 Health Gates

`doctor` must fail on:

- missing/incompatible CLI for the current platform;
- missing DB when an operation requires it;
- DB schema ahead of source, migration gap/duplicate/checksum drift;
- failed integrity/foreign-key check;
- repository/schema lineage mismatch;
- unsafe pending migration without backup;
- required Harness files missing.

### 8.2 Consistency Audit

`audit --strict` must check:

- story and decision Markdown ↔ DB parity;
- path existence, IDs, status, lane, verify command, and proof projection;
- open intake/task without terminal trace;
- completed task below trace/context/validation requirements;
- proof stale relative to commit/change scope;
- friction without disposition or backlog linkage;
- backlog lifecycle/outcome completeness;
- generated matrix freshness;
- CLI help ↔ compiled manifest ↔ `TOOL_REGISTRY.md` parity;
- root `AGENTS.md` ↔ installed-block shared core parity;
- installer manifest and ignore contract;
- portable capsule ↔ DB rebuild parity.

### 8.3 Maturity Claims

Recalibrate levels as follows until proof exists:

- H2: achieved locally, partial portably.
- H3: partial; observation exists but attribution/consistency gates are weak.
- H4: partial; commands exist, but proof freshness and closure enforcement do
  not.
- H5: not achieved until multiple implemented improvements have measured actual
  outcomes and recurrence/regression evidence.

Command existence is capability evidence, not outcome evidence.

## 9. Friction → Growth Protocol

For each friction occurrence:

```text
observe
  -> classify component/category/severity
  -> fingerprint and link task/trace
  -> choose disposition
       fixed-now
       backlog
       accepted-risk with expiry
       not-friction
  -> aggregate occurrences
  -> proposal with baseline and predicted metric
  -> human review when required
  -> implementation story + proof
  -> pending observation window
  -> actual outcome
  -> validated / ineffective / reverted
```

Proposal quality rules:

- Evidence names exact task/trace IDs and affected component.
- Suggested change names the smallest behavior/policy/tool delta.
- Predicted impact is measurable, not “improve reliability”.
- Validation includes a counterfactual or baseline when practical.
- Repeated open proposals are deduplicated.
- A proposal cannot be considered successful merely because code/docs changed.

Example:

```text
Friction: schema-lineage-cross-branch
Baseline: 1 current workspace where DB max=8 and source max=5; audit=0
Change: doctor rejects forward schema and binds DB lineage
Predicted: 100% of branch-switch incompatibilities fail before matrix query
Validation: fresh/main, stale-v1, ahead-v8, branch-switch, and rebuild E2E cases
Observation: next 5 Harness upgrades/branch switches
Outcome: no task reads incompatible DB; false positives = 0
```

## 10. Delivery Plan

Each phase is a separate story. Schema/source-hierarchy/validation changes are
high-risk and require decisions before implementation.

### Phase 0 — Freeze And Recover Current Truth

Risk: normal, with data caution.

- Back up the current DB and export a human-readable snapshot.
- Record the current `main` and `SYMPHONY` schema/state divergence.
- Decide which `US-004`/decision `0010` records belong only to `SYMPHONY`.
- Do not delete or rewrite current local state until recovery is reviewed.

Exit criteria:

- All recoverable records are attributable to a branch/source commit.
- A clean main-specific DB can be rebuilt without losing accepted main records.

### Phase 1 — P0 Safety Boundary

Risk: high-risk because migrations and durable state are touched.

- Add `doctor`, schema max/lineage/checksum checks, transactional migrations,
  backup, and forward-version failure.
- Make `init` ensure migration compatibility.
- Restrict `query sql` to read-only.
- Fix target `.gitignore` installation/managed block.
- Add stable error codes/JSON health output.

Exit criteria:

- The current version-8/main-version-5 state is rejected before matrix query.
- Existing old DB is migrated; ahead DB is not called up to date.
- Mutation through `query sql` fails.

### Phase 2 — Consistency And Rebuild

Risk: high-risk because source hierarchy and durable record semantics change.

- Add artifact index, docs/DB parity checks, safe import dry run, and rebuild.
- Introduce task capsules and update the SQLite durable-layer decision.
- Make fallback matrix generated/derived, not a competing editable truth.
- Add story/decision sync contracts and path/status checks.

Exit criteria:

- Fresh clone + `memory rebuild` reproduces critical stories, decisions,
  completed tasks, backlog state, and proof summaries.
- Any current docs/DB mismatch makes strict audit fail with repair guidance.

### Phase 3 — Atomic Task Lifecycle

Risk: high-risk because completion validation changes.

- Add `task start/status/finish/abandon`, final trace gate, proof links, context
  gate, and friction disposition.
- Reduce root/installed `AGENTS.md` to the start/status/finish command loop.
- Generate shared policy block from one source.

Exit criteria:

- A normal task cannot complete without intake, required trace tier, validation
  evidence/gap, and friction disposition.
- Read-only tasks can close with `files_changed=[]` without fake changes.
- High-risk missing decision/proof exits non-zero with exact remediation.

### Phase 4 — Measured Harness Growth

Risk: normal; high-risk per individual policy proposal.

- Add structured friction/fingerprints and backlog observation lifecycle.
- Improve `propose` aggregation/deduplication.
- Recalibrate maturity automatically from evidence.

Exit criteria:

- Similar phrasing maps to one reviewed friction fingerprint.
- Implemented improvements remain pending until actual outcome is measured.
- H5 is only claimable from closed outcome loops.

### Phase 5 — Ergonomics And Scale

Risk: normal.

- Add JSON/filtering/pagination, task ownership, concurrency tests, retention,
  platform diagnostics, and compact relevant-history retrieval.
- Optimize startup while keeping `doctor` under an agreed latency budget.

## 11. Required Decisions Before Implementation

Create accepted ADRs for:

1. SQLite authority versus version-controlled task capsules and rebuild rules.
2. Database schema lineage and branch/worktree isolation policy.
3. Task closure invariants and allowed override process.
4. Concern-specific source hierarchy and generated matrix status.
5. Trace/capsule privacy, retention, and redaction.

Do not encode these as trace-only decisions.

## 12. Validation Matrix

| Scenario | Expected result |
| --- | --- |
| Fresh install, no DB | create latest schema, doctor healthy, empty truthful audit |
| Existing old DB | backup + migrate transactionally + verify checksum |
| DB ahead of source | fail before query; show branch/lineage repair plan |
| Switch `SYMPHONY` → `main` | incompatible DB detected; no foreign story/decision used |
| Migration failure midway | rollback; version/checksum unchanged; backup available |
| `query sql DELETE ...` | rejected as non-read-only |
| Fresh brownfield rebuild | lane/path/status/verify/proof semantics preserved |
| Story Markdown missing from DB | strict audit fails and suggests safe sync direction |
| DB-only story | strict audit identifies branch/source provenance or orphan |
| Low-quality normal trace | task finish fails; no completed task state |
| Trace omits intake | standalone diagnostic only; cannot bypass lane gate |
| Proof passed on old commit | shown as stale, not pass |
| Backlog close without outcome | rejected or remains pending observation |
| Similar friction wording | grouped by reviewed fingerprint, not duplicated |
| Fresh clone/rebuild | critical read model equals exported canonical records |
| Installer upgrade | target product docs preserved; managed policy/schema updated |
| Installer and root policy | shared core parity test passes |
| Public CLI manifest | every public command, including `tool check`, appears once |
| Two agents same task | ownership conflict detected or explicitly handed off |
| Secret-like trace input | redaction warning/test prevents capsule promotion |

Minimum implementation proof per phase:

```bash
cargo fmt --all -- --check
cargo clippy -p harness-cli -- -D warnings
cargo test -p harness-cli
bash -n install.sh
bash -n install-harness-cli.sh
git diff --check
```

Add black-box CLI tests using temporary repositories/databases. Unit tests alone
cannot prove installer, branch, migration, capsule, and exit-code behavior.

## 13. Success Metrics

The loop is considered closed when all are true for a measured window:

- 100% of terminal tasks have an intake/task root and one final trace.
- 100% of normal/high-risk completed tasks meet required trace and proof gates.
- 100% of friction occurrences have a disposition.
- 0 unresolved schema-lineage, migration-checksum, or docs/DB parity errors.
- Fresh-clone rebuild reproduces 100% of critical semantic records.
- Proof shown as pass is tied to the current accepted commit/change scope.
- Every implemented Harness improvement has an actual outcome or remains
  explicitly pending observation.
- Repeated target friction decreases against its declared baseline; ineffective
  improvements are reverted or redesigned.
- Normal start/finish ergonomics use at most three primary Harness commands;
  details remain available behind focused subcommands.

## 14. Non-Goals

- Do not version the raw SQLite file.
- Do not store full prompts, raw command logs, secrets, or large artifacts in
  task capsules.
- Do not let `propose` autonomously rewrite Harness policy.
- Do not make every tiny task create a story.
- Do not replace product docs/decisions with database rows.
- Do not claim H5 from the existence of proposal commands.
- Do not repair the currently contaminated DB destructively in this proposal.

## 15. Immediate Recommended Next Action

Approve Phase 0 and the Phase 1 design direction only.

The first implementation story should be:

```text
Doctor rejects incompatible schema lineage before any operational query
```

Its acceptance criteria should cover the exact observed case:

```text
branch main
source migrations 1..5
local harness.db versions 1..8
current behavior: "Already up to date", audit 0
target behavior: non-zero doctor failure with source_max=5, db_max=8,
branch/worktree identity, backup/export guidance, and no matrix/proposal read
```

That slice prevents the Harness from learning or acting on untrustworthy state;
all later closure and evolution work depends on it.
