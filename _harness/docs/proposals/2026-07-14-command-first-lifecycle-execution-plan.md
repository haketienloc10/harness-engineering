# Execution Plan: Command-First Agent Lifecycle

Date: 2026-07-14

Status: Approved for phased implementation through CL-22; later phases remain
dependency-gated and require their own ready evidence

Plan ID: `CLP-001`

## Plan Amendments and Progress

| Work item | Status | Evidence / note |
| --- | --- | --- |
| CL-00 | completed | Recovery snapshot was verified without changing the original DB; see `docs/stories/CL-00-freeze-recover-baseline.md`. Case B is accepted in `docs/decisions/0010-main-schema-lineage-without-symphony.md`. |
| CL-01 | completed | Required decisions are accepted in `docs/decisions/0010..0016`. CL-10 may begin; migration numbering remains frozen until its schema manifest is in place. |
| CL-10 | completed | Main-lineage manifest `001..006`, read-only doctor, provenance and negative fixtures passed; source doctor correctly rejects the retained DB `001..008` as ahead. See `docs/stories/CL-10-doctor-and-schema-manifest.md`. |
| CL-13 | completed | Root `AGENTS.md` is now the canonical shared policy source; installer byte-parity and packaged-command checks are in `tests/installer_state_safety.sh`. |
| CL-20 | completed | Shadow mode, canonical flag aliases and typed lane/phase context policy are validated from `_harness/workflow.toml`. |
| CL-21 | completed | Pure checksummed context compiler, shared scoring path and 53 Rust tests passed; persistence/acknowledgement ownership moved to CL-40/41 by amendment. |
| CL-22 | completed | `workflow parity` validates accepted classification/context cases, command-manifest parity and the explicit delta disposition; packaged binary and installer fixtures pass. |

Baseline:

- Repository: `harness-engineering`
- Branch: `main`
- Commit: `ae580d7446b6d37a578fcf386f98f8612fe6cffe`
- CLI: `harness-cli 0.1.9`
- Source migrations: `001..005`
- Observed local DB migrations: `001..008`
- Related backlog: `#4 Establish schema-safe closed-loop Harness lifecycle`
- Related proposal:
  `_harness/docs/proposals/2026-07-13-closed-loop-harness-workflow.md`

## 1. Cách sử dụng plan này

Plan này là execution source cho việc chuyển Harness từ:

```text
agent đọc nhiều policy Markdown
  -> tự phân loại
  -> tự gọi nhiều lệnh rời rạc
  -> tự đồng bộ docs và DB
  -> tự quyết định task đã hoàn tất
```

sang:

```text
harness-cli task start
  -> doctor/ensure
  -> intake + lane + gates
  -> context manifest

agent thực hiện đúng context và proof plan

harness-cli task finish
  -> consistency + proof + approval + trace gates
  -> atomic closure
  -> portable task capsule
```

Agent thực hiện plan phải tuân thủ các quy tắc sau:

1. Không triển khai nhiều phase song song nếu phase sau phụ thuộc schema, CLI
   contract, artifact contract hoặc output của phase trước.
2. Mỗi work item `CL-xx` phải là một story riêng, có acceptance criteria,
   validation evidence và rollback note.
3. Không sửa hoặc xóa local `harness.db` hiện tại trước khi Phase 0 hoàn tất và
   recovery snapshot được human review.
4. Không tạo migration number mới trước khi ADR schema lineage quyết định
   canonical migration history.
5. Không compact `AGENTS.md` hoặc xóa policy files trước khi command-first path
   đạt parity và có compatibility tests.
6. Không đánh dấu một work item hoàn tất chỉ vì unit tests pass. Các black-box,
   migration, installer và crash-recovery cases liên quan cũng phải pass.
7. Sau mỗi story: cập nhật story Markdown, durable proof, plan progress table,
   matrix derived view và trace theo đúng command ordering.
8. Nếu phát hiện baseline khác plan, dừng work item đang phụ thuộc baseline,
   cập nhật phần `Plan Amendments`, và xin review nếu thay đổi architecture,
   source hierarchy, migration semantics hoặc completion gates.

## 2. Mục tiêu cuối cùng

Khi plan hoàn tất, một agent mới vào repository chỉ cần:

```bash
_harness/bin/harness-cli task start \
  --type change-request \
  --summary "Add account export"
```

CLI trả về:

- task ID và lifecycle state;
- computed lane, risk flags và lý do;
- exact `must_read`, `should_read`, `skip` paths;
- story/decision/approval requirements;
- relevant tool capability posture;
- proof plan và completion gates;
- explicit stop condition cho context retrieval.

Sau khi làm việc, agent chạy:

```bash
_harness/bin/harness-cli task finish \
  --id TASK-... \
  --outcome completed \
  --friction none
```

CLI chỉ đóng task nếu mọi invariant của lane được thỏa mãn. CLI tạo portable
task capsule khi policy/materiality yêu cầu và cập nhật operational state trong
một recoverable closure protocol.

## 3. Success criteria toàn chương trình

Chương trình chỉ hoàn tất khi tất cả tiêu chí sau được chứng minh:

- `100%` terminal tasks có một task root, lane và final trace.
- `100%` completed normal/high-risk tasks vượt đúng proof, approval, context và
  trace gates.
- `100%` material friction occurrences có disposition.
- `0` unresolved schema-lineage, migration-checksum hoặc docs/DB parity error.
- Fresh clone + `memory rebuild` tái tạo được toàn bộ critical semantic index.
- Proof hiển thị `pass` luôn gắn với current accepted commit/worktree state.
- Startup path bình thường yêu cầu tối đa một command và không bắt agent đọc
  toàn bộ policy docs.
- Normal task completion cần tối đa ba primary Harness commands:
  `task start`, `proof run` khi cần, `task finish`.
- Installed workflow/policy/template surface giảm ít nhất `70%` số dòng so với
  baseline khoảng `1,900` dòng.
- Không còn editable fallback matrix cạnh tranh với derived proof view.
- Không còn normal write path đi vòng qua lifecycle/application invariants.
- Fresh install, upgrade, branch switch và incompatible DB đều cho kết quả
  deterministic, machine-readable và có remediation rõ ràng.
- `100%` normal/high-risk và material tiny tasks có portable capsule; pure
  read-only tasks không tạo Git noise trừ khi phát hiện reusable decision,
  friction hoặc project lesson.

## 4. Non-negotiable invariants

### 4.1 Authority

| Concern | Canonical authority | Derived/indexed surface |
| --- | --- | --- |
| Current task intent | Current user instruction | Task row, trace |
| Product behavior | `docs/product/*` | Artifact index |
| Story intent/acceptance | `docs/stories/*` | Story DB projection |
| Durable decisions | `docs/decisions/*` | Decision DB projection |
| Harness policy | `_harness/workflow.toml` + accepted Harness ADRs | CLI context/gate output |
| Execution proof | Current `proof_run` records | Matrix query, task capsule |
| Portable task memory | `docs/tasks/*` capsules | Task/trace DB projection |
| Operational state | `harness.db` | Không được override semantic truth |

SQLite là local operational index/event store và phải rebuild được. Không
version raw SQLite file.

### 4.2 Safety

- Mọi operational command phải chạy shared preflight trước khi đọc/ghi DB.
- DB ahead-of-source, checksum mismatch hoặc lineage mismatch phải fail closed.
- Migrations phải backup-first và transactional.
- `query sql` phải read-only ở connection và statement level.
- Không overwrite artifact hiện có nếu không có explicit safe update mode.
- Paths từ config/artifact phải repo-relative, không chứa traversal và không
  resolve ra ngoài repository root.
- Shell verification chỉ chạy trong trusted repository; structured executable
  + args là default mới.
- Secrets, raw prompts và raw command logs không được đưa vào Git capsules.

### 4.3 Completion

- `completed` là state transition do `task finish` sở hữu, không phải free-text
  claim của standalone `trace`.
- Standalone trace không được đóng task.
- Missing proof/approval/decision không được biến thành warning rồi exit `0`.
- Mọi skipped gate cần structured reason, actor, expiry khi có, và follow-up.
- Task capsule phải tồn tại và checksum khớp trước khi DB commit terminal state
  khi effective policy đánh dấu capsule required. Pure read-only/non-material
  task có thể đóng DB-only nhưng phải lưu explicit `capsule_required=false` và
  reason.

### 4.4 Backward compatibility

- Không xóa legacy commands trong cùng release thêm command-first path.
- Legacy commands phải in deprecation warning nhưng giữ behavior trong ít nhất
  một compatibility release.
- Existing target product docs không bị installer overwrite.
- Existing DB được migrate hoặc rejected với recovery guidance; không silent
  reset.

## 5. Baseline cần bảo tồn và vấn đề cần sửa

Baseline tốt cần giữ:

- root `AGENTS.md` là entrypoint;
- risk lanes và hard gates;
- product/story/decision split;
- repository-local CLI;
- optional tool degrade behavior;
- trace/friction/intervention concepts;
- stateful command ordering;
- human approval cho high-risk direction.

Baseline lỗi cần sửa:

- source migrations `1..5` nhưng local DB có `1..8` và vẫn được đọc;
- `init` không migrate existing versioned DB;
- `query sql` có thể mutate;
- `audit 0/100` không kiểm tra schema/docs consistency;
- intake/trace không tạo một lifecycle root;
- trace dưới required tier vẫn được insert và command exit `0`;
- context scoring chỉ post-hoc;
- proof booleans độc lập với verification result và không có freshness;
- `harness.db` ignored nhưng bị xem như project memory;
- brownfield import làm mất lane/path/verify semantics;
- policy và CLI rules bị duplicate ở nhiều Markdown/Rust/installer surfaces;
- installer không quản lý `.gitignore` cho `harness.db`;
- không có owner/lease cho concurrent agents.

## 6. Target installed surface

Kết quả cuối cùng trong target repository:

```text
AGENTS.md
README.md
.harness-id                  # tracked stable repository UUID

_harness/
  workflow.toml
  bin/
    harness-cli
    harness-cli.ps1          # chỉ khi platform package hỗ trợ
  scripts/
    schema/
      manifest.toml
      *.sql
  templates/
    story.md
    decision.md
    initiative.md            # optional; chỉ cho multi-story spec

docs/
  product/
  stories/
  decisions/
  tasks/

harness.db                   # ignored local operational state
harness.db.backups/          # ignored, retention-controlled
```

`.harness-id` được tạo một lần khi cài/ensure repository mới, phải được commit,
không bị installer overwrite và không chứa secret. Fresh clone dùng cùng ID;
worktree identity được ghi riêng. Nếu file thiếu trong brownfield repo, `ensure`
preview ID trước khi tạo và `doctor --strict` yêu cầu commit/disposition.

Source-only Harness design docs được giữ trong source repository nhưng không
cài vào target:

```text
docs/harness-internal/
  audit-model.md
  component-taxonomy.md
  maturity-model.md
  proposals/
```

## 7. Target CLI contract

### 7.1 Primary commands

```text
doctor [--strict] [--json]
workflow validate|explain [--json]
task start|status|refresh|finish|block|resume|abandon|approve
task context acknowledge
proof run|query
memory check|rebuild|export
friction add|resolve|query
audit [--strict] [--json]
```

Legacy commands tồn tại trong compatibility window:

```text
init, migrate, intake, story add/update/verify, trace,
score-trace, score-context, import brownfield
```

### 7.2 Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Operation completed and all requested gates passed |
| `2` | CLI usage or config validation error |
| `3` | Unsafe durable state detected by doctor |
| `4` | Migration/backup/rollback failure |
| `5` | Task completion gate failed |
| `6` | Artifact/docs/DB consistency failure |
| `7` | Proof command failed or proof is stale |
| `8` | Task ownership/concurrency conflict |
| `9` | Human approval required or missing |
| `10` | Unexpected internal error |

CLI JSON output phải chứa ổn định:

```json
{
  "ok": false,
  "code": "DB_AHEAD_OF_SOURCE",
  "message": "...",
  "details": {},
  "remediation": ["..."]
}
```

Human-readable output và JSON phải được render từ cùng domain result; không duy
trì hai bộ rule riêng.

### 7.3 `doctor`

Checks tối thiểu, theo thứ tự:

1. Locate repo root và reject ambiguous/nested root.
2. Detect platform/architecture và CLI compatibility.
3. Validate required installed paths.
4. Parse `_harness/workflow.toml` và validate supported policy version.
5. Inventory source migrations; detect duplicate, gap, invalid filename.
6. Open DB safely; nếu thiếu, report `DB_MISSING` mà không tự tạo trong pure
   `doctor` mode.
7. Run SQLite integrity and foreign-key checks.
8. Read DB lineage/version/checksums.
9. Compare DB/source lineage and versions.
10. Detect pending migrations and backup requirement.
11. Validate repository identity, worktree path, branch và current commit.
12. Check required artifact paths and managed ignore rules.
13. Return health report; `--strict` treats consistency warnings as failures.

`doctor` tuyệt đối không mutate state. `ensure` behavior nằm trong `task start`
hoặc compatibility `init`.

### 7.4 `task start`

Proposed interface:

```bash
_harness/bin/harness-cli task start \
  --type change-request \
  --summary "Add account export" \
  --flags "public-contract,weak-proof" \
  --behavior-bearing auto \
  --owner codex \
  --json
```

Arguments:

- `--type`: required canonical input type.
- `--summary`: required, normalized whitespace, minimum length.
- `--flags`: optional explicit flags; CLI may add inferred flags but must name
  inference evidence.
- `--behavior-bearing`: `auto|yes|no`, default `auto`.
- `--story`: attach existing story ID.
- `--story new`: atomically allocate/scaffold a new story when policy requires
  one; combine with `--story-title`. The CLI creates metadata/template only,
  never invents acceptance criteria.
- `--owner`: agent/session identity; default from environment only when stable.
- `--resume`: explicit task ID; never silently resume by fuzzy summary.
- `--lane`: optional override only with `--lane-reason`; override cannot lower a
  hard gate without linked approval.
- `--json`: stable machine output.

Execution order:

1. Locate root.
2. Run doctor preflight.
3. If DB missing/behind and safe: backup as needed, migrate, rerun doctor.
4. Load and validate workflow policy.
5. Normalize input type/flags. Caller/agent supplies semantic flags; any CLI
   inference must be deterministic, advisory and include evidence. CLI must not
   pretend it understands arbitrary prompt semantics from summary text alone.
6. Compute recommended lane and gates from explicit/validated flags.
7. Check owner conflicts/open tasks on same story/worktree.
8. Create intake + task root in one DB transaction.
9. Attach or require story according to behavior-bearing result.
10. Run `tool check`; return only capabilities relevant to selected gates.
11. Build context manifest from policy + artifacts + task state.
12. Store manifest checksum and selected context requirements.
13. Print task ID, context and next command.

Start result schema:

```json
{
  "task_id": "TASK-000123",
  "status": "open",
  "lane": "normal",
  "lane_reason": ["public-contract", "weak-proof"],
  "story": {"required": true, "id": null},
  "must_read": ["docs/product/account-export.md"],
  "should_read": ["docs/decisions/0012-export-format.md"],
  "skip": ["docs/stories/unrelated.md"],
  "gates": ["story", "proof:unit", "proof:integration", "trace:standard"],
  "tools": [],
  "stop_condition": "Stop retrieval after all must_read paths are loaded."
}
```

### 7.5 `task status`

Phải trả:

- state và allowed next transitions;
- owner/session/worktree;
- story/decision/approval links;
- required vs satisfied gates;
- proof current/stale/fail status;
- context manifest compliance;
- unresolved friction;
- staged or orphaned capsule state;
- exact remediation commands.

Context compliance is an attestation, not invisible observation. CLI cannot
prove a model read a file. Agent records reads using either:

```bash
_harness/bin/harness-cli task context acknowledge \
  --id TASK-000123 \
  --read docs/product/account-export.md
```

or supplies a response file/list to `task finish --read ...`. The CLI validates
paths against the stored manifest and records timestamp/agent. Git-derived
changed files come from start/end snapshots; agents do not manually claim those
when Git can provide them.

`task refresh --id ...` is required if effective policy checksum, branch,
worktree, attached story or affected paths change materially after start. It
recomputes context/gates, shows the delta, requires explicit acceptance, and
never silently drops an already-required gate.

### 7.6 `proof run`

Preferred interface:

```bash
_harness/bin/harness-cli proof run \
  --task TASK-000123 \
  --story US-012 \
  --layer integration \
  -- cargo test -p harness-cli
```

Rules:

- Arguments sau `--` được lưu thành executable + argv, không join thành shell
  string.
- Shell mode chỉ qua explicit `--shell` và warning trusted-repository.
- Command chạy từ canonical repo root hoặc declared safe cwd.
- Capture start/end time, exit code, `HEAD`, branch, dirty fingerprint, CLI
  version, platform, command digest và output artifact hash.
- Raw stdout/stderr lưu ở ignored evidence path với size limit; capsule chỉ lưu
  result + hash + short summary.
- Failed run vẫn được ghi append-only.
- Initial freshness rule: pass chỉ current khi exact `HEAD` và dirty fingerprint
  khớp. Path-scoped freshness là later optimization, không nằm trong first
  cutover.
- `not_applicable` cần story validation plan hoặc explicit reason; không dùng
  false boolean.

Proof states:

```text
not_required | not_run | running | pass | fail | stale | not_applicable
```

### 7.7 `task finish`

Interface:

```bash
_harness/bin/harness-cli task finish \
  --id TASK-000123 \
  --outcome completed \
  --friction none \
  --read docs/product/account-export.md
```

Preflight không mutate:

1. Doctor strict health.
2. Task exists, owner matches hoặc handoff/approval exists.
3. Transition từ current state sang requested outcome hợp lệ.
4. Lane và risk flags complete.
5. Story requirement satisfied hoặc valid non-behavior reason recorded.
6. Product/story/decision artifacts parse được và DB projections match.
7. Required approvals present.
8. Required proof layers current và pass/not-applicable hợp lệ.
9. Effective policy checksum still matches start/last refresh.
10. Context requirements have explicit acknowledgement hoặc approved skip.
11. Trace fields đạt tier.
12. Every friction has disposition.
13. Worktree/commit state captured.
14. Capsule requirement/materiality is resolved.
15. Capsule content passes redaction and schema validation when required.

Recoverable closure protocol:

1. Determine capsule requirement from lane/materiality/friction/decision state.
2. Render capsule in memory when required.
3. Write capsule to same-filesystem temporary path.
4. Flush file; validate parse/checksum.
5. Begin SQLite `IMMEDIATE` transaction.
6. Insert final trace, links, closure nonce và set task `closing`.
7. Atomically rename staged capsule to final path when required.
8. Record final artifact checksum/path or explicit no-capsule reason; set
   terminal task state.
9. Commit DB transaction.
10. Rerun read-only consistency check.
11. Print completed result.

Crash cases:

- Crash trước rename: DB transaction rollback; temp file cleanup on next doctor.
- Crash sau rename trước DB commit: capsule orphan được doctor phát hiện; resume
  finish có thể attach nếu nonce/checksum match, không tự xóa.
- DB terminal nhưng required capsule missing/checksum mismatch: doctor hard
  failure.
- Repeated `task finish` với same nonce: idempotently return existing result.
- Non-material no-capsule task: no file staging occurs; DB transaction still
  writes final trace, explicit materiality decision and terminal state.

### 7.8 Other transitions

```text
open -> in_progress -> blocked -> in_progress
open|in_progress|blocked -> abandoned
open|in_progress -> failed
in_progress -> closing -> completed
```

- `task block`: reason + missing authority/external condition + next action.
- `task resume`: explicit ID; validates ownership and health again.
- `task abandon`: requires reason; does not fake completion proof.
- `task approve`: records claimed human/reviewer approval, gate, source message
  reference và scope. CLI cannot prove a human identity; it must not claim
  cryptographic authorization.

## 8. `_harness/workflow.toml` contract

`workflow.toml` là single machine-readable policy source. CLI refuses unknown
major version và reports unknown keys theo strict mode.

Minimum shape:

```toml
policy_version = "1.0"
policy_id = "agent-first-default"

[repository]
product_docs = "docs/product"
stories = "docs/stories"
decisions = "docs/decisions"
tasks = "docs/tasks"

[lanes.tiny]
trace_tier = "minimal"
story = "when_behavior_bearing"
proof = ["quick"]
capsule = "when_material"

[lanes.normal]
trace_tier = "standard"
story = "when_behavior_bearing"
proof = ["declared-story-plan"]
capsule = "required"

[lanes.high_risk]
trace_tier = "detailed"
story = "required"
proof = ["declared-validation-plan"]
capsule = "required"

[classification]
normal_min_flags = 2
high_risk_min_flags = 4
hard_gates = [
  "auth",
  "authorization",
  "data-migration",
  "audit-security",
  "external-provider",
  "weaken-validation",
]

[approvals]
required_for = [
  "architecture-direction",
  "source-hierarchy",
  "risk-policy",
  "weaken-validation",
  "destructive-data-action",
]

[[context.rules]]
id = "schema-change"
when_paths = ["_harness/scripts/schema/**"]
must_read = ["docs/decisions/0004-sqlite-durable-layer.md"]

[[context.rules]]
id = "cli-distribution"
when_paths = ["crates/harness-cli/**", "_harness/bin/**", "install.sh"]
must_read = ["docs/decisions/0005-prebuilt-rust-harness-cli.md"]

[friction]
allowed_dispositions = [
  "fixed-now",
  "backlog",
  "accepted-risk",
  "not-friction",
]
```

Policy ownership rules:

- Config defines deterministic lane/gate/context behavior.
- Accepted ADRs explain why config has those values.
- `AGENTS.md` chỉ hướng agent tới lifecycle commands và human gates.
- CLI help được generate/render từ typed command definitions.
- Không duplicate lane thresholds trong Rust; Rust chỉ validate/execute config.
- Installer embeds one generated AGENTS block from a tracked template, không
  chứa hand-maintained duplicate literal.

## 9. Durable data model

Không viết SQL migration cho đến khi Phase 0 chọn lineage. Target logical
records:

### 9.1 `harness_meta`

```text
repository_id
schema_lineage
created_by_cli_version
created_at
last_doctor_at
last_doctor_commit
last_doctor_worktree
policy_id
policy_version
```

### 9.2 `migration_history`

```text
version primary key
name
checksum
applied_at
cli_version
source_commit
```

Legacy `schema_version` được giữ trong compatibility window nhưng không còn đủ
để tuyên bố schema healthy.

### 9.3 `task`

```text
id primary key                # TASK-000123
intake_id not null unique
created_at, updated_at, closed_at
status
outcome
risk_lane
behavior_bearing
lane_override_reason
summary
owner
session_id
repository_id
worktree
branch
start_commit
end_commit
start_dirty_fingerprint
end_dirty_fingerprint
context_manifest_json
context_manifest_checksum
closure_nonce
capsule_path
capsule_checksum
capsule_required
capsule_omission_reason
```

Constraints:

- terminal outcome required iff status terminal;
- one open task per explicit owner/session/story policy;
- allowed status values enforced;
- immutable start provenance;
- terminal task requires capsule metadata when `capsule_required=1`; otherwise
  requires a non-empty omission reason.

### 9.4 Link tables

```text
task_story(task_id, story_id, role)
task_decision(task_id, decision_id, role)
task_backlog(task_id, backlog_id, role)
task_approval(task_id, gate, source, evidence, scope, created_at)
```

### 9.5 `proof_run`

```text
id
task_id
story_id nullable
layer
state
executable
argv_json
shell_mode
cwd
started_at, finished_at
exit_code
head_commit
branch
dirty_fingerprint
cli_version
platform
command_digest
stdout_path/hash
stderr_path/hash
artifact_path/hash
summary
```

Proof rows append-only. Derived matrix không nhận direct boolean writes.

### 9.6 Trace changes

- Add `task_id` foreign key.
- Add `trace_kind`: `attempt|final|diagnostic`.
- Require final trace exactly once per terminal task.
- Store arrays as validated JSON, không comma-split text.
- Add repository/commit/worktree/CLI provenance.
- Standalone diagnostic traces không tham gia completion.

### 9.7 Artifact index

```text
artifact_type
artifact_id
path
checksum
schema_version
status
source_commit
indexed_at
```

Unique `(artifact_type, artifact_id)` và `path`. Index là rebuildable projection,
không phải semantic source.

### 9.8 Structured friction

```text
friction(id, fingerprint, component, category, severity, title, status)
task_friction(task_id, friction_id, trace_id, occurrence_text, disposition,
              disposition_reason, backlog_id, expires_at)
```

### 9.9 Lifecycle constraints

Application layer phải validate transitions trước SQL. DB constraints bảo vệ
invalid enum/null state. Không dựa riêng vào CLI argument parsing.

## 10. Portable artifact schemas

### 10.1 Story frontmatter

Story mới dùng YAML frontmatter versioned:

```yaml
---
schema: harness/story/v1
id: US-012
title: Account Export
status: planned
lane: normal
product_docs:
  - docs/product/account-export.md
proof_plan:
  unit: required
  integration: required
  e2e: not_applicable
  platform: not_applicable
---
```

Body tối thiểu:

```text
Contract
Acceptance Criteria
Scope / Non-Goals
Design and Risk Notes
Validation Plan
Evidence
```

High-risk story dùng cùng một file và thêm sections:

```text
Approval Gates
Data/Migration/Rollback
Security/Authorization
Observability/Audit
Execution Phases
```

Chỉ tách directory packet khi story vượt agreed size hoặc nhiều agents cần
independent ownership.

### 10.2 Decision frontmatter

```yaml
---
schema: harness/decision/v1
id: 0012-command-first-lifecycle
title: Command-First Lifecycle Authority
status: accepted
date: 2026-07-14
---
```

### 10.3 Task capsule

Path:

```text
docs/tasks/YYYY/MM/TASK-000123-short-slug.md
```

Minimum frontmatter:

```yaml
---
schema: harness/task-capsule/v1
task_id: TASK-000123
date: 2026-07-14
lane: normal
outcome: completed
story_ids: [US-012]
decision_ids: []
start_commit: abc123
end_commit: def456
dirty_fingerprint: sha256:...
proof_runs:
  - id: 42
    layer: integration
    result: pass
    command_digest: sha256:...
friction: []
backlog_ids: []
---
```

Body:

- Outcome.
- Changed surfaces.
- Decisions/non-goals.
- Validation evidence and explicit gaps.
- Friction disposition.
- Reusable lesson for future agents.

Không chứa raw prompt, secret, full logs hoặc machine-specific absolute paths.

### 10.4 Parser migration

- Parser v1 phải đọc current heading-based stories/decisions để compatibility.
- `memory check` reports legacy artifacts nhưng không rewrite mặc định.
- Explicit `memory migrate --dry-run` preview frontmatter conversion.
- Actual conversion là separate reviewed story; preserve body và Git diff nhỏ.

## 11. Migration lineage rule

Trước migration mới, Phase 0 phải chọn một trong hai trường hợp:

Migration checksum algorithm phải được khóa trong ADR/manifest: SHA-256 của
UTF-8 SQL sau khi bỏ BOM và normalize line endings thành LF, nhưng giữ nguyên
mọi byte nội dung khác. Release tooling generate/verify manifest; applied
migration file không được sửa. Semantic correction dùng migration mới.

### Case A — `SYMPHONY` migrations `006..008` được chấp nhận vào main

1. Verify exact file content/checksums.
2. Bring the same migrations into canonical main history.
3. Existing compatible DB may be retained after checksum/repository checks.
4. First new migration của plan là `009-*`.

### Case B — `SYMPHONY` migrations không thuộc main

1. Quarantine current ahead DB; không downgrade in place.
2. Rebuild a main-lineage DB from canonical Git artifacts/export.
3. Canonical main next migration là `006-*`.
4. DB chứa foreign `006..008` luôn fail lineage/checksum và cần explicit
   `HARNESS_DB` separation hoặc recovery import.

Không được chọn migration number chỉ để “vượt qua version 8”. Version monotonic
không thay thế lineage/checksum.

## 12. Delivery dependency graph

```text
CL-00 Freeze/recover baseline
  -> CL-01 Accept required ADRs
      -> CL-10 Doctor + schema manifest
          -> CL-11 Safe ensure/migrate
          -> CL-12 Read-only SQL boundary
          -> CL-13 Installer state safety
              -> CL-20 workflow.toml schema
                  -> CL-21 context compiler
                  -> CL-22 policy parity tests
                      -> CL-30 artifact schemas/index
                          -> CL-31 memory check/rebuild
                          -> CL-32 task capsule renderer
                              -> CL-40 task root/lifecycle schema
                                  -> CL-41 task start/status/approval
                                  -> CL-42 proof run/freshness
                                      -> CL-43 task finish/crash recovery
                                          -> CL-50 compact AGENTS/install block
                                          -> CL-51 consolidate templates
                                          -> CL-52 deprecate/remove runtime docs
                                              -> CL-60 structured friction
                                              -> CL-61 evidence-based maturity
                                                  -> CL-70 release/upgrade proof
```

`CL-11`, `CL-12` có thể phát triển song song sau `CL-10`, nhưng integration phải
serialize. Các story khác phải theo dependency graph.

## 13. Phase 0 — Freeze, recover, decide

### CL-00 — Freeze and export current durable truth

Risk: high-risk data handling, read-only except backup/export artifacts.

Files/artifacts:

- backup outside normal DB path;
- human-readable inventory under an explicitly reviewed evidence directory;
- no source policy change.

Steps:

1. Record current branch, commit, CLI version, platform and worktree status.
2. Copy `harness.db`, WAL and SHM consistently. If WAL exists, use SQLite-safe
   backup mechanism; plain copy only after checkpoint/closed connection.
3. Hash backup files.
4. Export schema versions, table/index list and schema SQL.
5. Run integrity and foreign-key checks.
6. Export story, decision, backlog, intake, trace, intervention and tool rows.
7. Attribute DB-only `US-004`, decision `0010`, tables and migrations `006..008`
   to branch/commit if possible.
8. Compare docs IDs/status/path with DB rows.
9. Record secrets/privacy review for exported traces.
10. Human reviews which records belong to main vs `SYMPHONY`.
11. Verify or create stable tracked `.harness-id`; never derive repository
    identity only from mutable remote URL, absolute path or current branch.

Acceptance:

- Backup hashes recorded and restore tested on a temp path.
- No current data deleted/rewritten.
- Every DB-only canonical candidate has provenance or is marked unknown.
- Main rebuild input set is explicitly listed.

Rollback: delete only newly created export copies after review; original DB is
untouched.

### CL-01 — Accept required architecture decisions

Create and accept ADRs before implementation:

1. SQLite authority, Git task capsules and rebuild rules.
2. Schema lineage, migration checksums and branch/worktree isolation.
3. Task lifecycle/closure invariants and override policy.
4. Concern-specific source hierarchy and generated matrix status.
5. Workflow policy authority (`workflow.toml`) and CLI rendering.
6. Trace/capsule privacy, retention and redaction.
7. Proof execution trust model and shell command policy.

Acceptance:

- ADRs are Git-tracked and indexed in a clean/rebuilt DB.
- No ADR exists only as trace text.
- Migration lineage Case A/B is selected.
- Human explicitly approves source hierarchy and validation-gate direction.

Stop condition: without accepted ADRs, do not start CL-10.

## 14. Phase 1 — Safety boundary

### CL-10 — Doctor and schema manifest

Likely files:

- `_harness/scripts/schema/manifest.toml`
- `crates/harness-cli/src/domain.rs` or extracted `domain/health.rs`
- `crates/harness-cli/src/application.rs`
- `crates/harness-cli/src/infrastructure.rs` or extracted migrations module
- `crates/harness-cli/src/interface.rs`
- black-box test fixtures.

Implementation:

1. Add typed source migration inventory.
2. Add manifest lineage/version/checksum definitions.
3. Add read-only DB inspection.
4. Implement all doctor checks from section 7.3.
5. Add stable domain error codes and JSON/human renderers.
6. Ensure every operational service entrypoint can invoke shared preflight.
7. `doctor` itself remains non-mutating.

Acceptance scenarios:

- no DB;
- healthy latest DB;
- legacy version-1 DB;
- DB behind source;
- DB ahead of source;
- migration gap/duplicate;
- checksum mismatch;
- lineage mismatch;
- corrupt DB;
- foreign-key violation;
- wrong platform binary;
- missing required path;
- invalid workflow policy version.

Rollback: command can be removed without DB mutation; manifest remains inert.

### CL-11 — Idempotent ensure and safe migration

Implementation:

1. Compatibility `init` delegates to `ensure`.
2. `ensure` runs doctor, determines create/migrate/reject.
3. Backup naming includes timestamp, DB version, lineage and checksum.
4. Set backup retention; never delete the newest known-good backup.
5. Apply each migration transactionally.
6. Record checksum/CLI/source commit in migration history.
7. Rerun integrity, foreign-key and doctor after migration.
8. Restore guidance is printed on failure; automatic restore only if proven safe
   and tested.
9. Reject ahead/foreign lineage rather than calling it current.

Acceptance:

- fresh DB reaches latest source schema;
- old DB backs up then migrates;
- migration failure rolls back schema/history;
- backup restores successfully;
- ahead DB returns exit `3` without writes;
- second ensure is idempotent.

### CL-12 — Read-only SQL boundary

Implementation:

1. Open a SQLite read-only connection.
2. Reject multiple statements.
3. Use SQLite statement-readonly/authorizer checks where available.
4. Permit safe `SELECT`, read-only `WITH`, and diagnostic PRAGMA allowlist.
5. Reject DML, DDL, ATTACH, writable PRAGMA, VACUUM and extension loading.
6. Remove arbitrary SQL from normal compiled write capabilities.
7. If admin write SQL is retained, place behind explicit separate command,
   backup, force/approval and audit record; default recommendation is omit it.

Acceptance: mutation corpus leaves copied DB byte/logical state unchanged.

### CL-13 — Installer state safety

Implementation:

1. Install a managed `.gitignore` block, preserving user entries.
2. Ignore `harness.db`, WAL/SHM, backups, temp capsules and evidence logs.
3. Generate AGENTS shared block from one tracked template.
4. Detect platform/architecture before installing CLI binary.
5. Installer reports payload updated, DB untouched, pending ensure, or
   incompatibility.
6. Add installer manifest parity tests.
7. Create `.harness-id` only when missing, preserve it on every upgrade, and
   report that it should be committed.

Acceptance:

- fresh install tracks no local state;
- upgrade preserves product docs and DB;
- repeated install is idempotent;
- user `.gitignore` content survives;
- stable `.harness-id` survives reinstall, clone and worktree use;
- unsupported platform gives explicit remediation.

Phase 1 exit gate:

- Current main/source-5 + DB-8 case fails before matrix/audit/propose reads.
- No normal query path can mutate DB.
- Old compatible DB migrates safely.

## 15. Phase 2 — Policy compiler and context manifest

### CL-20 — Introduce typed `workflow.toml`

Implementation:

1. Define versioned config schema and serde types.
2. Validate enum values, lane thresholds, hard gates, paths and globs.
3. Reject unknown major version; warn/error unknown keys by mode.
4. Implement `workflow validate` and `workflow explain`.
5. Add canonical config fixture and malformed config corpus.
6. Keep old Markdown authoritative during this story; no cutover yet.
7. Add materiality/capsule policy and context acknowledgement rules.

Acceptance:

- Config roundtrip stable.
- Equivalent inputs produce deterministic lane/gates.
- Invalid path traversal and inconsistent thresholds rejected.

Amended ownership: materiality values remain typed lane policy here; task-bound
acknowledgement state and refresh behavior are owned by CL-40/CL-41 under the
2026-07-14 CL-22-unblock amendment.

### CL-21 — Context compiler

Implementation:

1. Convert phase/lane/retrieval rules to typed policy rules.
2. Collect changed/affected paths and linked artifacts.
3. Produce ordered, deduplicated must/should/skip lists.
4. Include reason per context entry.
5. Include stop condition and token-budget hint.
6. Persist manifest/checksum on task.
7. Make scoring evaluate the stored manifest, not hardcoded Markdown paths.
8. Implement explicit acknowledgement and `task refresh` delta behavior.

Acceptance:

- Golden tests cover tiny/normal/high-risk across intake/planning/work/finish.
- Schema/CLI/provider/public-contract triggers select exact expected context.
- Unrelated docs do not appear in must-read.

Amended scope: CL-21 completes the pure deterministic manifest and checksum.
Item 6 persistence is owned by CL-40; item 8 acknowledgement/refresh is owned
by CL-41. Item 7 evaluates the compiled manifest for legacy `score-context`
without requiring task records. See the 2026-07-14 CL-22-unblock amendment.

### CL-22 — Policy parity and drift gate

Implementation:

1. Encode current accepted rules into config.
2. Build comparison fixture from current policy cases.
3. Report intentional deltas explicitly.
4. Test root AGENTS block, installed block, config and CLI command manifest.
5. Do not compact files yet; mark config as shadow mode.

Exit gate:

- Shadow command output matches accepted current policy or an ADR-approved delta.
- CLI/config is ready to become authority.

## 16. Phase 3 — Portable semantic memory

### CL-30 — Artifact schemas and index

Implementation:

1. Define story/decision/capsule schemas.
2. Implement legacy and v1 parsers.
3. Validate IDs, paths, status, lane and references.
4. Create artifact index projection.
5. Add `story check`, `decision check`, `memory check --dry-run`.
6. Never rewrite docs during check.

### CL-31 — Deterministic memory rebuild

Implementation:

1. Read canonical stories, decisions and capsules.
2. Produce conflict report before write.
3. Rebuild into a new temp DB, never mutate current DB in place.
4. Validate rebuilt DB with doctor/audit.
5. Atomically switch only after explicit command and backup.
6. Record import provenance/checksums.
7. Retire ambiguous brownfield upsert behavior.

Acceptance:

- Fresh clone rebuild preserves ID, lane, status, paths and proof summaries.
- Duplicate IDs/path conflicts fail with remediation.
- Rebuild repeated twice produces equivalent logical state.

### CL-32 — Capsule renderer/redaction

Implementation:

1. Versioned capsule renderer/parser.
2. Redact configured secret patterns and absolute machine paths.
3. Reject unsafe/oversized fields.
4. Stage, validate, checksum and atomically rename.
5. Add orphan/stale capsule detection.

Phase 3 exit gate:

- `harness.db` can be recreated from Git-tracked canonical records for critical
  project memory.
- Static matrix is no longer needed for recovery.

## 17. Phase 4 — Atomic task lifecycle

### CL-40 — Task root and lifecycle schema

Implementation:

1. Add task/link/approval tables and constraints.
2. Backfill legacy intake/trace associations conservatively.
3. Do not fabricate task closure where linkage is unknown.
4. Add transition domain tests.
5. Add owner/session/worktree conflict checks.

### CL-41 — `task start`, `status`, `approve`, `block`, `resume`, `abandon`

Implementation follows sections 7.4, 7.5 and 7.8.

Special cases:

- Read-only question: behavior-bearing false, no story; resolve capsule from
  materiality and create no Git capsule when there is no reusable outcome.
- Tiny copy edit: story optional, quick proof required.
- Existing story: attach without duplicate creation.
- Multiple stories: one primary plus explicit secondary links.
- Same agent retry: explicit resume.
- Different agent takeover: handoff/approval record.

### CL-42 — Proof run and freshness

Implementation follows section 7.6.

Additional gates:

- No direct `story update --unit 1` on normal path.
- Matrix derives state from proof runs and story applicability.
- Failed proof cannot be overwritten by boolean; later pass remains separate run.
- Dirty fingerprint algorithm documented and cross-platform tested.

### CL-43 — `task finish` and crash recovery

Implementation follows section 7.7.

Required failure tests:

- missing intake/task root;
- missing story;
- missing decision/approval;
- stale/failing/missing proof;
- insufficient trace tier;
- unmet context;
- unresolved friction;
- docs/DB mismatch;
- owner conflict;
- capsule write/rename failure;
- DB commit failure;
- repeated finish/idempotency;
- orphan capsule reconciliation.

Phase 4 exit gate:

- No completed normal/high-risk task can bypass configured gates.
- Read-only task closes without fake changed file.
- Completion returns non-zero and structured remediation on every gate failure.

## 18. Phase 5 — Cutover and payload compaction

### CL-50 — Compact AGENTS entrypoint

Target content only includes:

1. What Harness is and scope boundary.
2. Run `task start` before edits.
3. Follow returned context/gates.
4. Run `task finish` before final response.
5. Human approval conditions.
6. CLI-missing emergency fallback.

No duplicated lane tables, trace field tables, command ordering catalog or
source hierarchy prose. Generated installed block must match tracked template.

### CL-51 — Consolidate templates

Actions:

- merge normal/high-risk story templates into one progressive template;
- merge validation report into story validation/evidence and proof runs;
- replace spec intake template with optional initiative template;
- retain decision template;
- add capsule template owned by renderer, not manually copied.

### CL-52 — Remove files from runtime workflow

| Current file | Final disposition |
| --- | --- |
| `_harness/HARNESS.md` | Remove after compatibility release; policy in config/CLI |
| `_harness/FEATURE_INTAKE.md` | Remove; rules in config |
| `_harness/CONTEXT_RULES.md` | Remove; context compiler |
| `_harness/TRACE_SPEC.md` | Move source-only or generate `task finish --explain` |
| `_harness/TOOL_REGISTRY.md` | Move source-only; CLI help/config owns runtime contract |
| `_harness/TEST_MATRIX.md` | Delete editable copy; query derived view |
| `_harness/ARCHITECTURE.md` | Remove from generic payload; create product architecture docs on demand |
| `_harness/IMPROVEMENT_PROTOCOL.md` | Move source-only; runtime rules in config |
| `_harness/HARNESS_AUDIT.md` | Source-only; `audit --explain` for runtime |
| `_harness/HARNESS_COMPONENTS.md` | Source-only |
| `_harness/HARNESS_MATURITY.md` | Source-only/generated evidence |
| `_harness/README.md` | Delete redundant index |
| `templates/validation-report.md` | Merge/remove |
| `templates/high-risk-story/*` | Merge into progressive story template |

Cutover sequence:

1. Release N: command-first available, old workflow remains; warnings off.
2. Observe internal tasks and fix parity gaps.
3. Release N+1: AGENTS uses command-first; legacy docs marked deprecated;
   legacy commands warn.
4. Installer excludes source-only/deprecated docs but upgrade preserves unknown
   user files safely.
5. Release N+2: remove deprecated runtime docs/commands only after usage and
   rebuild evidence confirm safety.

Phase 5 exit gate:

- Fresh target payload contains only target installed surface.
- Existing target upgrade has no lost product/user file.
- Agent startup reads no redundant policy file.

## 19. Phase 6 — Learning loop and evidence maturity

### CL-60 — Structured friction and backlog lifecycle

Implement fingerprint/category/severity/disposition model and lifecycle:

```text
proposed -> accepted -> in_progress -> implemented_pending_observation
  -> validated | ineffective | reverted
```

Require baseline, predicted metric, observation window and actual outcome.

### CL-61 — Evidence-based audit/maturity

- Separate doctor health from audit debt.
- Audit unknown coverage explicitly; zero findings is not “perfect” if checks
  were not run.
- Derive maturity from observed gates/outcomes, not command existence.
- Require multiple measured improvements before H5.

## 20. Phase 7 — Release and operational proof

### CL-70 — Release/upgrade qualification

Required environments:

- fresh Linux target;
- existing old-schema target;
- DB ahead-of-source target;
- branch-switch target;
- dirty worktree;
- CLI missing/wrong platform;
- fresh clone + memory rebuild;
- installer rerun/upgrade;
- two concurrent agent sessions.

Release evidence:

- Cargo format, clippy, unit/integration tests;
- black-box CLI tests;
- installer shell tests;
- migration/backup/restore tests;
- crash-recovery tests;
- payload manifest snapshot;
- AGENTS template parity;
- command manifest parity;
- docs/DB roundtrip;
- startup latency benchmark;
- five to ten dogfood task capsules.

## 21. Test matrix chi tiết

### Health/migration

- Missing DB.
- Empty zero-byte DB.
- Unversioned legacy DB.
- Every supported old version.
- Latest version.
- DB one version behind.
- DB ahead.
- Missing source migration.
- Duplicate migration number.
- Migration checksum changed after apply.
- Foreign lineage.
- Corrupt main DB.
- WAL/SHM present.
- Backup directory unwritable.
- Disk full during backup/migration.
- SQL failure midway.
- Post-migration integrity failure.
- Concurrent migration attempt.

### Policy/context

- Missing config.
- Unsupported major version.
- Unknown key strict/non-strict.
- Invalid lane threshold.
- Invalid hard gate.
- Path traversal/glob escaping repo.
- Tiny/normal/high-risk classification boundaries.
- Every hard gate.
- Lane override up/down.
- Relevant/irrelevant docs dedupe/order.
- Context stop condition.
- Policy checksum changes during open task.

### Task lifecycle

- New tiny read-only task.
- Tiny behavior change.
- Normal existing/new story.
- High-risk approval missing/present.
- Explicit resume.
- Duplicate open task.
- Owner conflict and handoff.
- Block/resume/abandon/failed/completed transitions.
- Finish twice.
- Finish from invalid state.
- Policy changes between start/finish.
- Policy refresh accepts added gates and never silently removes satisfied or
  previously required gates.
- Branch/worktree changes between start/finish.

### Proof

- Structured argv with spaces/commas/unicode.
- Shell mode warning.
- Pass/fail/timeout/signal.
- Missing executable.
- Output size limit.
- Secret-like output handling.
- Exact clean commit freshness.
- Dirty state freshness.
- Commit change -> stale.
- Not applicable with/without reason.
- Multiple layers/runs.
- Failed then passed history.

### Artifacts/memory

- Legacy story/decision parse.
- V1 frontmatter parse.
- Duplicate ID/path.
- Missing referenced product doc.
- Invalid status/lane/schema.
- DB-only/docs-only record.
- Rebuild dry-run.
- Rebuild idempotency.
- Capsule redaction.
- Capsule path collision.
- Orphan capsule.
- DB terminal without capsule.
- Fresh clone parity.

### Installer/payload

- Empty target.
- Existing AGENTS without block.
- Existing/current/stale block.
- Existing product docs.
- Existing `.gitignore`.
- Existing `_harness` and DB.
- Deprecated files from prior release.
- Unsupported platform.
- Manifest accuracy.
- Repeated install.
- No source-only docs in target.

### Security/permissions

- `query sql` DML/DDL/ATTACH/PRAGMA/VACUUM/multi-statement rejection.
- Artifact symlink/path traversal.
- Config path escape.
- Shell proof explicit trust gate.
- Secret redaction and capsule rejection.
- Missing/forged approval metadata is not described as verified identity.

## 22. Validation commands

Minimum per implementation story:

```bash
cargo fmt --all -- --check
cargo clippy -p harness-cli -- -D warnings
cargo test -p harness-cli
bash -n install.sh
bash -n install-harness-cli.sh
git diff --check
```

Additional commands must be added as scripts exist:

```text
black-box CLI suite against temp repositories/databases
fresh install smoke
upgrade smoke
memory rebuild roundtrip
migration failure/restore suite
task lifecycle E2E suite
capsule crash-recovery suite
payload manifest snapshot
```

Do not add fake package/CI scripts before their implementation exists.

## 23. Rollout telemetry and observation

Measure before and after:

- policy files read per task;
- Harness commands per task;
- startup latency p50/p95;
- task finish gate failures by category;
- context must-read compliance;
- over-read count;
- docs/DB drift count;
- stale proof count;
- friction occurrences and dispositions;
- installer/upgrade failures;
- manual human corrections per five related tasks.

Initial observation window: minimum ten dogfood tasks, including at least:

- three tiny;
- four normal;
- two high-risk;
- one blocked/resumed;
- one fresh clone/rebuild;
- one installer upgrade.

## 24. Stop conditions and human gates

Stop and request human direction when:

- lineage Case A/B cannot be resolved;
- recovery requires deleting/downgrading original DB;
- accepted ADRs conflict;
- validation requirements would be weakened;
- source hierarchy changes beyond section 4;
- CLI would overwrite product/user files;
- task capsule privacy/redaction requirements are unclear;
- platform distribution scope expands beyond approved targets;
- a migration cannot be made rollback-safe;
- a compatibility break must ship earlier than the agreed window.

## 25. Agent handoff checklist per story

Before implementation:

- Confirm dependencies in section 12 are completed.
- Read the story, relevant ADRs and exact files to change.
- Run doctor and capture baseline.
- Check open tasks/owners.
- Confirm no unrelated dirty changes will be overwritten.

During implementation:

- Keep producer/consumer commands sequential.
- Add tests with the implementation, including negative paths.
- Update progress and decisions when scope changes.
- Preserve backward compatibility unless story explicitly owns cutover.

Before completion:

- Run required validation.
- Run black-box cases owned by the story.
- Verify migrations/installer on temp state only.
- Update durable story/proof evidence.
- Rerun matrix/doctor/audit as applicable.
- Record trace and friction disposition.
- Update plan progress table.

Handoff must state:

- exact files changed;
- schema/API/CLI contract changes;
- validation commands and results;
- remaining gaps;
- rollback procedure;
- next unblocked story.

## 26. Progress table

Agents update this table only after evidence exists.

| Work item | Status | Dependency | Evidence | Next action |
| --- | --- | --- | --- | --- |
| CL-00 Freeze/export | completed | none | `docs/stories/CL-00-freeze-recover-baseline.md` | Preserve recovery snapshot |
| CL-01 ADRs | completed | CL-00 | `docs/decisions/0010..0016` | Continue CL-10 |
| CL-10 Doctor | completed | CL-01 | `docs/stories/CL-10-doctor-and-schema-manifest.md`; 40 CLI tests plus installer syntax checks | Start CL-11 |
| CL-11 Ensure/migrate | completed | CL-10 | `docs/stories/CL-11-ensure-safe-migration.md`; 45 CLI tests, installer syntax and actual ahead-DB exit `3` | Start CL-12 |
| CL-12 Read-only SQL | completed | CL-10 | `docs/stories/CL-12-read-only-sql-boundary.md`; 46 CLI tests | Start CL-13 |
| CL-13 Installer safety | completed | CL-10 | `docs/stories/CL-13-installer-state-safety.md`; installer black-box smoke passed | Start CL-20 |
| CL-20 workflow.toml | completed | Phase 1 | `docs/stories/CL-20-typed-workflow-policy.md`; typed config, CLI and 53 tests | Start CL-21 |
| CL-21 Context compiler | completed | CL-20 | `docs/stories/CL-21-context-compiler.md`; pure typed compiler and 53 tests | Persist with CL-40; acknowledge/refresh with CL-41 |
| CL-22 Policy parity | completed | CL-21 | `docs/stories/CL-22-policy-parity-and-drift-gate.md`; `workflow parity`, ADR 0017, 54 Rust tests and installer black-box proof | Start CL-30 |
| CL-30 Artifact schemas | completed | CL-22 | Migration 007, legacy/v1 read-only artifact checks, duplicate/reference fixtures, 56 Rust tests and installer proof | Start CL-31 |
| CL-31 Memory rebuild | completed | CL-30 | Typed temporary rebuild, artifact/story/decision projection, logical digest, conflict gate and explicit safe apply are proven | Start CL-32 |
| CL-32 Capsule renderer | completed | CL-30 | Versioned renderer/parser, checksum, redaction, collision refusal and orphan detection are proven | Start CL-40 |
| CL-40 Task schema | completed | Phase 3 | Transition graph and terminal SQLite constraints are tested; the retained DB remains rejected as ahead of source | CL-41 may use the validated root |
| CL-41 Task start/status | in_progress | CL-40 | Atomic start, policy lane/context, acknowledgement, approval, explicit refresh, owner enforcement/handoff and primary/secondary story links are fixture-tested | Add session/lease identity |
| CL-42 Proof run | in_progress | CL-40 | Structured `proof run/query` append/expose executable+argv, pass/fail, HEAD and dirty fingerprint; status compares freshness | Add branch/output provenance, artifact-scoped freshness and remove direct boolean normal path |
| CL-43 Task finish | completed | CL-41, CL-42, CL-32 | Required-capsule staging/atomic rename, deterministic closure nonce, rollback/retry recovery, terminal doctor invariants and normal/high-risk gates are unit-proven | CL-50/CL-60 may begin |
| CL-50 Compact AGENTS | completed | CL-43 | Canonical/install shared `AGENTS.md` is command-first only and installer byte parity passed | CL-51/CL-60 may begin |
| CL-51 Templates | completed | CL-43 | Progressive story template owns high-risk expansion, validation and rollback; compatibility templates are deprecated; policy/parity and installer checks pass | CL-52 may begin |
| CL-52 Remove runtime docs | completed | CL-50, CL-51 | Workflow context no longer points at source-only docs; installer excludes them and upgrade-safety checks pass | CL-60 may begin |
| CL-60 Structured friction | completed | CL-43 | Migration 009, `friction add/resolve/query`, deterministic fingerprint and terminal task gate are unit/black-box proven | CL-61 may begin |
| CL-61 Maturity | in_progress | CL-60 | Audit now exposes unobserved material friction and explicit coverage scope; measured multi-improvement maturity threshold remains | Add outcome-derived maturity report |
| CL-70 Release proof | blocked | all prior | — | — |

## 27. Definition of done cho plan

Plan được coi là executed hoàn toàn khi:

- progress table không còn `not_started`/`blocked` cho in-scope work;
- every story has durable proof and a terminal task capsule;
- required ADRs are accepted;
- target CLI and installed surface match sections 6–10;
- all test matrices and release environments pass;
- compatibility removals follow the agreed release window;
- observation metrics meet section 3 or deviations are explicitly accepted;
- backlog #4 is closed with measured actual outcome, not merely code shipped.

## 28. Plan amendments

Mọi amendment phải ghi:

```text
date
author/agent
affected work items
old assumption
new evidence
decision/approval reference
validation and rollback impact
```

Không rewrite lịch sử amendment. Nếu amendment thay source hierarchy, lineage,
completion invariant hoặc privacy policy, tạo/update ADR trước.

### 2026-07-14 — CL-10 preflight surface

- Author/agent: Codex
- Affected work items: CL-10 (with workflow configuration retained as its
  doctor-validation input; CL-20 remains the owner of typed policy execution).
- Old assumption: doctor could establish health from migration versions alone.
- New evidence: a version match cannot prove safe operation without a
  manifest checksum, schema lineage, repository root, required payload and
  supported workflow-policy version.
- Decision reference: `0010-main-schema-lineage-without-symphony`,
  `0014-workflow-policy-authority`.
- Validation and rollback impact: read-only fixtures cover source/DB failure
  modes and doctor has no write path; remove source-only doctor/manifest code
  to roll back without modifying a local DB.

### 2026-07-14 — Unblock CL-22 by separating pure compilation from lifecycle state

- Author/agent: Codex, approved by the current user instruction.
- Affected work items: CL-13, CL-20, CL-21, CL-22, CL-40, CL-41 and CL-50.
- Old assumption: CL-21 could require task manifest persistence,
  acknowledgement and refresh before CL-22, while task lifecycle schema and
  commands remained downstream of CL-22. AGENTS shared-source parity was also
  deferred to CL-50 even though CL-22 requires it.
- New evidence: those ownership assignments create the cycles
  `CL-21 -> CL-22 -> CL-30/32 -> CL-40/41 -> CL-21` and
  `CL-22 -> CL-43 -> CL-50 -> CL-22`. The packaged binary also lagged the
  source command definition, and current Markdown flag spellings did not share
  a canonical vocabulary with config.
- Execution worktree note: the approved preparation and CL-22 start point are
  on `feature-rework`; the original `main` baseline remains recovery/provenance
  context and is not rewritten by this amendment.
- Decision/approval reference: accepted ADR 0014 continues to make
  `_harness/workflow.toml` the future machine authority; current user approval
  accepts this delivery-only scope correction and authorizes starting CL-22 in
  the next session. No lifecycle completion or source-hierarchy invariant
  changes.
- New ownership: CL-21 is a pure, deterministic context compiler. CL-40 owns
  storing its manifest/checksum; CL-41 owns acknowledgement and refresh delta
  semantics. Root `AGENTS.md` is the canonical tracked shared policy source;
  CL-13 owns byte-parity installation, while CL-50 may only compact it after
  command-first cutover. Migration 006 task/proof tables remain inert until
  their owning lifecycle stories expose validated application paths.
- Validation and rollback impact: lane/phase golden tests, AGENTS byte parity,
  compiled/tracked command manifest parity and packaged-binary installer tests
  are required. The previously missing tracked `.harness-id` is introduced as
  part of the approved preparation; it must not be regenerated on reinstall.
  Rollback removes the new shadow/compiler surfaces without mutating task or
  database state; repository identity is preserved once published.

## 29. Immediate next action

Start `CL-22 policy parity and drift gate` from
`docs/stories/CL-22-policy-parity-and-drift-gate.md`. Keep config in shadow
mode, parse the prepared policy fixture, and resolve or explicitly disposition
the `one-flag-code-impact` delta before authority cutover. The retained
`harness.db` must remain rejected as `DB_AHEAD_OF_SOURCE`; CL-22 uses pure
workflow commands and temporary test databases only.
