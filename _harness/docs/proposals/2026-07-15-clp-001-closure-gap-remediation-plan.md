# CLP-001 Closure Gap Remediation Plan

Date: 2026-07-15

Status: Planned — execution has not started

Plan ID: `CLP-001-R1`

Parent plans:

- `_harness/docs/proposals/2026-07-14-command-first-lifecycle-execution-plan.md`
- `_harness/docs/proposals/2026-07-15-clp-001-full-closure-plan.md`

Audit source: `TASK-000022`, trace `#30`, and
`docs/tasks/2026/07/TASK-000022-audited-clp-001-closure-and-found.md`

## 1. Objective

Close the contract and evidence gaps discovered after CLP-001 was marked
`Fully closed`, without rewriting historical tasks, traces, approvals, proof
runs, audit dispositions, or recovery databases.

The remediation is complete only when:

- the documented one-command `task start` path works as written and returns the
  complete computed lifecycle contract;
- `task status` and `task finish` expose the completion information and
  remediation promised by the parent plan;
- semantic-memory validation and rebuild include story packets and nested task
  capsules, and can recreate the critical portable projection from a fresh
  clone;
- strict audit coverage is derived from real checks and becomes unknown or
  failing when semantic-memory parity has not been demonstrated;
- every original CLP-001 work item has a truthful canonical story/evidence
  disposition without fabricated historical execution;
- the complete release and negative-test ladders pass from a clean committed
  Linux `x86_64` HEAD;
- a terminal lifecycle task closes through `task finish`, followed by
  read-only doctor, audit, memory, task-status, and Git-state checks.

This plan corrects the implementation and its evidence. It must not silently
weaken the parent plan so the current code appears compliant.

## 2. Verified Starting State

The independent audit on 2026-07-15 established the following at commit
`f6946f999932954d518e619940c0c6b68bf8be32`:

### 2.1 Healthy release and operational state

- `cargo fmt --all -- --check` passes.
- `cargo clippy --workspace -- -D warnings` passes.
- `cargo test --workspace` passes.
- `bash tests/release_qualification.sh all` passes.
- `doctor --strict --json` reports `HEALTHY` at source and database migrations
  `001..012` on Linux `x86_64`.
- `audit --strict --json` reports `AUDIT_CLEAR`; H5 is `achieved` and the two
  approved historical dispositions remain visible.
- `TASK-000018..21` are completed with released leases and their configured
  context, approval, proof, trace, and capsule gates satisfied.
- Canonical backlog successor `#2` is `implemented` with a measured actual
  outcome.

These facts must remain true throughout remediation.

### 2.2 Reproduced command-contract gaps

The exact parent-plan startup example:

```bash
_harness/bin/harness-cli task start \
  --type change-request \
  --summary "Add account export"
```

exits `2` because `--behavior-bearing <yes|no>` is mandatory.

With all currently required arguments, `task start --json` returns only task,
status, owner/session, and lease fields. It omits computed lane reasons,
story/decision/approval requirements, `must_read`, `should_read`, `skip`,
proof/completion gates, relevant tools, stop condition, and next command.

`task status` exposes counts and latest-proof freshness but omits the complete
required/satisfied gate view, decision/approval links, unresolved friction,
capsule state, and exact remediation commands. JSON parse/usage failures also
do not consistently use the stable structured error envelope promised by the
parent plan.

The parent-plan `task finish` example also omits the currently mandatory
`--trace` argument, so the supported finish contract and the documented
three-command path are not yet aligned.

### 2.3 Reproduced semantic-memory gaps

Current artifact discovery reads only direct `.md` children of
`docs/stories`, `docs/decisions`, and `docs/tasks`.

Consequences observed by `TASK-000022`:

- 14 canonical story packet directories are skipped;
- 13 task capsules under `docs/tasks/2026/07` are skipped by
  `memory check/rebuild` discovery;
- current `artifact_index` contains 12 story rows and 19 decision rows, but no
  capsule rows;
- `memory rebuild --dry-run --json` reports success with 31 artifacts and does
  not prove task/capsule projection parity;
- the JSON `temp_schema_version` is hard-coded to `7` for created/migrated
  candidates even though current canonical migrations are `001..012`;
- `tests/release_qualification.sh` proves that capsule files are byte-identical
  in the clone, but does not prove that they were parsed, indexed, or projected
  into rebuilt operational state.

### 2.4 Reproduced audit-coverage gap

When a release-qualification observation exists, audit marks
`portable capsule to fresh-rebuild parity` and related release coverage as
covered. That conclusion is not currently tied to a named semantic-parity
assertion or proof artifact. Therefore `unknown_coverage=[]` can be reported
while packet and nested-capsule discovery is incomplete.

### 2.5 Formal story/evidence gaps

- No canonical `CL-01` story artifact exists, although the parent plan requires
  each `CL-xx` work item to be a separate story.
- Several packet-based CL stories are absent from the current story projection.
- Early retained stories have no linked completed task capsule/current
  structured proof satisfying the literal plan-level definition of done.
- Existing historical tasks and traces must not be fabricated to repair these
  gaps retrospectively.

## 3. Scope and Non-Goals

### In scope

- CLI contract and structured output for `task start`, `task status`, and
  `task finish`.
- Deterministic behavior-bearing `auto` semantics based on typed intake and
  explicit flags, never arbitrary summary-language inference.
- Recursive, safe artifact discovery and packet identity/checksum rules.
- Story, decision, capsule, and critical task-summary projection in fresh
  rebuilds.
- Truthful audit coverage and negative coverage fixtures.
- Current-evidence reconciliation for CLP-001 stories and capsules.
- Source and packaged CLI parity, installer safety, migration/rebuild safety,
  and terminal release qualification on Linux `x86_64`.

### Out of scope

- Platform expansion beyond Linux `x86_64`.
- Destructive database repair, direct operational SQL, or historical-row
  rewriting.
- Revoking or hiding approved audit dispositions `#1` and `#2`.
- Early removal of legacy commands, parsers, `schema_version`, or other N+2
  compatibility surfaces.
- Inferring product behavior from arbitrary natural-language summaries.
- Reclassifying old execution as observed when it was not observed.
- Weakening proof freshness, capsule requirements, strict audit, or doctor
  health gates.

## 4. Human Gates and Required Decisions

Pause for explicit approval before implementation of the following:

1. `architecture-direction`: canonical story-packet identity and aggregate
   checksum/projection semantics.
2. `source-hierarchy`: whether a packet is represented by its directory plus
   canonical `overview.md`, or by a new manifest/component model.
3. `risk-policy`: what constitutes proven versus unknown audit coverage.
4. `lifecycle-contract`: deterministic `behavior-bearing=auto` rules and the
   supported final-trace ownership of `task finish`.
5. Any migration or backup-first application to the retained operational DB.

Recommended directions, subject to approval:

- Normalize canonical input types across space, underscore, and hyphen forms.
- Make `--behavior-bearing` optional with default `auto`.
- Resolve `auto` conservatively from typed input type, explicit flags, and
  linked story only; return the reasons. Do not inspect summary prose.
- Treat `overview.md` as the packet identity document and compute a
  deterministic aggregate checksum over all allowed packet files in sorted
  repo-relative order.
- Introduce a backward-compatible richer capsule schema for rebuildable
  critical task summaries. Existing v1 capsules remain readable but missing
  coverage must be explicit until upgraded from observed durable data.
- Make `task finish` own final closure: it may select a qualifying trace rooted
  in the task intake or accept an explicit trace, but the minimal documented
  path must be executable and unambiguous.

## 5. Delivery Sequence

Execute sequentially in one worktree. Do not retain multiple live worktree
leases. Each behavior-bearing work item must have its own lifecycle task,
current structured proof, trace, and capsule.

| Order | Work item | Suggested lane | Responsibility |
| ---: | --- | --- | --- |
| 0 | `CGR-00` bootstrap | tiny | Register CL-72 and freeze negative fixtures |
| 1 | `CGR-10` lifecycle command contract | normal | Complete start/status/finish interfaces and error JSON |
| 2 | `CGR-20` portable semantic rebuild | high-risk | Recursive packets/capsules, projection, schema/report correctness |
| 3 | `CGR-30` audit truth and evidence reconciliation | high-risk | Coverage registry, negative audit gates, CLP-001 story evidence |
| 4 | `CGR-40` terminal requalification | high-risk | Clean-HEAD release, strict gates, final closure amendment |

Create one remediation story:

`CL-72 — Command lifecycle and portable-memory closure corrections`

Use a tiny non-behavioral bootstrap task to register CL-72 and its progressive
artifacts before starting behavior-bearing work.

## 6. CGR-00 — Bootstrap and Freeze Failing Contracts

### Work

1. Start a tiny non-behavioral task and acknowledge its complete context.
2. Add and link CL-72 through `harness-cli story add`.
3. Create `overview.md`, `design.md`, `execplan.md`, and `validation.md` for
   CL-72.
4. Capture the current failing command and memory cases as black-box fixtures
   without weakening existing passing suites.
5. Record current counts and hashes for story packets, capsules,
   `artifact_index`, source migrations, operational migrations, and retained
   database backup.
6. Record required architecture/risk-policy questions for human approval.

### Required negative fixtures

- Exact two-argument parent-plan `task start` invocation.
- Full `task start --json` schema assertion.
- `task status --json` gate/remediation schema assertion.
- JSON usage/error envelope assertion.
- Nested valid capsule that must be discovered.
- Nested corrupted capsule that must fail check/rebuild.
- Valid packet story that must be discovered and projected.
- Duplicate packet ID and unsafe symlink/path cases.
- Rebuild whose capsule/task projection differs from the source artifacts.
- Audit run with semantic parity proof absent, stale, or failed.

### Exit gate

Fixtures fail for the intended missing contract, not because of test setup.
The bootstrap task finishes through the CLI with truthful non-behavioral
evidence.

## 7. CGR-10 — Lifecycle Command Contract

### 7.1 `task start`

Implement and prove:

- input type normalization for canonical space/underscore/hyphen spellings;
- optional `--behavior-bearing auto|yes|no`, default `auto`;
- deterministic auto-classification reasons from typed inputs and explicit
  flags only;
- the exact minimal parent-plan invocation succeeds;
- stable human and JSON rendering from one domain result;
- JSON includes task/status, lane and reasons, behavior classification,
  story/decision/approval requirements, `must_read`, `should_read`, `skip`,
  stop condition, relevant tools, proof/completion gates, and exact next
  command;
- no created task is left ownerless or semantically ambiguous under the
  approved ownership policy.

### 7.2 `task status`

Return, in human and JSON modes:

- state, ownership, session, worktree, lease, and transitions;
- all linked stories, decisions, approvals, backlog items, and friction;
- required and satisfied gates with per-gate state;
- proof layer state and all freshness dimensions;
- full context manifest plus acknowledgement/skip state;
- capsule required/staged/orphaned/final state;
- exact ordered remediation commands for every unmet gate.

### 7.3 `task finish`

Align the documented minimal path and implementation:

- define whether a qualifying rooted trace is auto-selected or supplied;
- keep explicit `--trace` available for deterministic recovery/idempotency;
- never synthesize trace content;
- reject zero or multiple ambiguous qualifying final traces with remediation;
- ensure the parent-plan three-primary-command path is executable;
- preserve atomic capsule/DB closure and all existing Phase 4 failure cases.

### 7.4 Structured errors

When `--json` is present, parse, usage, policy, preflight, and domain failures
must use the stable error envelope and documented exit codes. Add a pre-parse
or renderer strategy for argument-parser failures rather than emitting an
unstructured usage block.

### Exit gate

Source, packaged, tracked command manifest, help snapshots, and black-box
behavior agree. Existing legacy command behavior remains within the N+2
compatibility contract.

## 8. CGR-20 — Portable Semantic Rebuild

This work is high-risk because it changes canonical artifact interpretation
and rebuild semantics. Obtain the required architecture/source-hierarchy
approval first.

### 8.1 Safe recursive discovery

- Walk configured artifact roots recursively in deterministic sorted order.
- Reject symlink escape, traversal, duplicate identity, case collision, and
  unsafe file type.
- Ignore documented non-artifact files only through explicit rules.
- Discover nested `docs/tasks/YYYY/MM/*.md` capsules.
- Discover story packets without treating every supporting Markdown file as a
  separate story.

### 8.2 Packet contract

Implement the approved packet identity model. At minimum it must preserve:

- story ID, title, status, lane, and canonical packet path;
- ordered component paths and deterministic aggregate checksum;
- acceptance, design, execution, validation, rollback, and evidence content;
- duplicate/collision detection across single-file and packet stories;
- compatibility with existing single-file legacy and v1 stories.

### 8.3 Capsule contract and projection

- Recursively validate every capsule schema and content checksum.
- Define a richer backward-compatible capsule representation containing enough
  observed data to rebuild critical terminal task summaries and links.
- Render/upgrade capsules only from current durable records and immutable proof
  artifacts; never invent historical fields.
- Project capsule entries into `artifact_index`.
- Rebuild the supported task/story/trace/proof summary projection required by
  accepted ADRs, or explicitly report non-rebuildable legacy fields as unknown.
- Preserve v1 read compatibility through the N+2 window.

### 8.4 Rebuild correctness

- Initialize the candidate at the actual current canonical schema, not a
  hard-coded version.
- Report the version read from the candidate database.
- Run both `doctor --strict` and `audit --strict` against the candidate.
- Compare source artifact counts, IDs, paths, statuses, links, checksums, and
  critical proof summaries with the rebuilt projection.
- Produce a machine-readable parity report with mismatch details.
- Keep `--dry-run` non-destructive and `--apply` backup-first, validated, and
  atomic.

### Exit gate

A fresh clone rebuild includes all canonical packet stories and nested task
capsules, produces identical logical digests twice, and fails when any required
artifact is omitted or corrupted.

## 9. CGR-30 — Audit Truth and Evidence Reconciliation

### 9.1 Coverage registry

Replace release-observation inference with named check results. Each coverage
item records:

- check ID and version;
- state: `pass|fail|unknown|not_applicable`;
- command/proof run and output/artifact hash;
- HEAD, branch, dirty fingerprint, and freshness;
- scope and measured counts;
- failure/unknown remediation.

`audit --strict` passes only when every required check is current and `pass`,
and there are no unresolved findings or unknown coverage.

### 9.2 Negative audit cases

Strict audit must fail or report unknown when:

- recursive capsule validation was not executed;
- a nested capsule is missing, invalid, or unprojected;
- a packet story is missing or its aggregate checksum differs;
- fresh rebuild parity compares only file-copy hashes;
- the parity proof is stale for HEAD/branch/dirty state;
- reported artifact counts differ from discovered counts;
- candidate schema/report version differs from actual database state.

### 9.3 Story and evidence reconciliation

- Add a truthful canonical CL-01 story artifact describing the ADR package and
  current validation; do not claim a historical task that never existed.
- Register every retained packet story through supported CLI/memory paths.
- Revalidate each original CLP-001 work item against current behavior.
- Link current reconciliation proofs and a current terminal capsule to every
  retained story lacking one.
- Use an explicit durable historical/non-applicable disposition only when
  current proof cannot truthfully establish the contract; require human
  approval and keep it visible.
- Rebuild `artifact_index` through the supported command path and eliminate
  stale status/path/checksum projection.

### Exit gate

Audit reports exact discovered/projected/rebuilt counts. No CLP-001 story is
missing, silently skipped, stale, or falsely described as historically
executed.

## 10. CGR-40 — Terminal Requalification

Start only after CGR-10 through CGR-30 are committed and the worktree is clean.
This task owns terminal proof and documentation reconciliation, not new product
behavior.

### Required structured proof ladder

```text
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
bash -n install.sh
bash -n install-harness-cli.sh
bash -n tests/installer_state_safety.sh
bash -n tests/release_qualification.sh
bash tests/installer_state_safety.sh
bash tests/release_qualification.sh state
bash tests/release_qualification.sh distribution
bash tests/release_qualification.sh all
_harness/bin/harness-cli workflow parity --json
_harness/bin/harness-cli memory check --dry-run --json
_harness/bin/harness-cli memory rebuild --dry-run --json
_harness/bin/harness-cli memory capsule check --json
_harness/bin/harness-cli doctor --strict --json
_harness/bin/harness-cli audit --strict --json
git diff --check
```

Also run the exact command-contract and negative semantic-memory/audit suites
introduced by this plan against both source and packaged CLIs.

### Terminal actions

1. Confirm every required context entry is acknowledged.
2. Confirm all approvals are present and scoped.
3. Confirm all proof layers are current at the clean committed HEAD.
4. Confirm every friction occurrence has a terminal disposition.
5. Update CL-72 validation with exact task IDs, proof IDs/hashes, counts,
   rollback, and compatibility obligations.
6. Append a corrective amendment to both parent plans. Do not rewrite the prior
   closure history; state that the later audit found and then closed gaps.
7. Mark checkboxes complete only from direct current evidence.
8. Record a detailed rooted trace and render the required capsule.
9. Finish through `harness-cli task finish`.
10. Run post-finish read-only task status, doctor, audit, memory parity, capsule
    check, and Git status.

## 11. Terminal Acceptance Checklist

Do not mark `CLP-001-R1` completed until every item is true:

- [ ] Exact minimal `task start` command succeeds.
- [ ] `task start --json` returns the complete documented lifecycle contract.
- [ ] `task status` returns all gate, link, friction, capsule, and remediation
      details.
- [ ] Supported minimal `task finish` path is aligned with documentation and
      preserves trace truthfulness.
- [ ] All `--json` errors use the stable structured envelope.
- [ ] All 14 current story packets are discovered and validated.
- [ ] All nested task capsules are discovered, validated, and indexed.
- [ ] Fresh rebuild projects the approved critical story/task/capsule memory.
- [ ] Rebuild JSON reports the actual candidate schema version.
- [ ] Source and rebuilt artifact counts, IDs, paths, status, checksums, links,
      and required proof summaries match.
- [ ] Negative packet/capsule/parity fixtures fail closed.
- [ ] Audit coverage is backed by named current proof and becomes
      fail/unknown when that proof is absent, stale, or incomplete.
- [ ] CL-01 and every original CLP-001 work item have truthful canonical story
      evidence or an explicitly approved visible disposition.
- [ ] No historical task, trace, proof, or approval was fabricated.
- [ ] Source, packaged, installed, and tracked command manifests agree.
- [ ] Installer state safety and full release qualification pass.
- [ ] `doctor --strict --json` is `HEALTHY` at the current canonical schema.
- [ ] `audit --strict --json` is clear with zero unresolved and zero unknown
      required coverage.
- [ ] Accepted historical dispositions remain visible and effective.
- [ ] H5 remains `achieved` from observed evidence.
- [ ] Terminal task is completed with released lease, full context,
      approvals, fresh proof, detailed trace, and committed capsule.
- [ ] Final worktree is clean.

## 12. Rollback

- CLI contract: revert source, packaged binary, command manifest, help
  snapshots, and tests together.
- Artifact semantics: revert parser/index changes and restore the pre-change DB
  backup with the prior packaged binary. Do not manually rewrite the index or
  task rows.
- Capsule upgrade: preserve old capsules and generated upgrade evidence; never
  destroy the only portable record.
- Audit coverage: revert registry/report code together, but do not describe
  unmeasured checks as covered during rollback.
- Story reconciliation: revert Git artifacts and use supported CLI commands for
  durable correction; preserve historical tasks/traces/proofs.
- Terminal documentation: append a rollback amendment rather than deleting the
  audit and remediation history.

## 13. Session Handoff Prompt

Suggested prompt for the implementation session:

> Execute
> `_harness/docs/proposals/2026-07-15-clp-001-closure-gap-remediation-plan.md`
> sequentially. Start with CGR-00 through the command-first lifecycle. Preserve
> all historical evidence, stop for the listed architecture/source-hierarchy/
> risk-policy approvals, add negative fixtures before fixes, use structured
> proof runs, and do not mark CLP-001 fully closed again until every terminal
> acceptance item has direct clean-HEAD evidence.

## 14. Immediate Next Action

In the next session, start the tiny CGR-00 bootstrap task, register CL-72, add
the progressive story packet, and freeze the failing command/memory/audit
fixtures before changing implementation.
