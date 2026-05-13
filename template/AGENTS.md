# AGENTS.md

Repository này đã cài **Harness** để điều phối AI-assisted development bằng artifact, role policy, lifecycle state, và template-based subagent orchestration.

File này là bootstrap instruction của target repository. Quy trình chi tiết nằm trong `.harness/guides/*`, `.harness/workflows/*`, `.harness/subagents/*`, và `.harness/scripts/*`. Chỉ load tài liệu liên quan đến task hiện tại.

## Bootstrap Rules

- Reply theo ngôn ngữ người dùng; giữ code, command, path, API name, logs, schema keys, package names, và identifiers ở dạng gốc.
- Trước non-trivial work, đọc `.harness/HARNESS_SKILLS.md`, chọn skill/guide liên quan theo name, description, trigger conditions, và chỉ load phần cần thiết.
- Không load toàn bộ `.harness/guides/*`, `.harness/subagents/*`, hoặc `.harness/codebase/*` theo mặc định.
- Dùng `run.yaml` làm authoritative lifecycle state.
- Không sửa application code trước khi Contract Reviewer approve contract và `run.yaml` cho phép Generator.
- Evaluator phải độc lập với Generator và phải có evidence thật.
- Không kết thúc bằng generic “Suggested Next Steps” nếu bước tiếp theo có thể biểu diễn bằng Harness lifecycle routing hoặc executor dispatch.

## Repository Boundary

`.harness/` là workflow infrastructure của target repository. Nó chứa guides, templates, scripts, run records, project adapter files, codebase cache, subagent templates, workflows, và backlog cho AI-assisted development.

`.harness/` không phải application source tree.

Khi làm implementation task, role phù hợp phải inspect source code, tests, runtime behaviour, và architecture thực tế của target repository bên ngoài `.harness/`.

Coordinator không được dùng câu trên như quyền để tự inspect source nhằm implement, debug, repair, verify, hoặc review thay lifecycle roles.

## Context Loading

Trước implementation work không tầm thường:

1. Đọc `.harness/HARNESS_SKILLS.md`.
2. Chọn skill/guide liên quan.
3. Nếu cần project-level context, đọc các file liên quan trong `.harness/project/*`.
4. Nếu cần source-navigation hoặc impact context, đọc `.harness/codebase/CODEBASE_INDEX.md` và chỉ các `.harness/codebase/*` docs liên quan.
5. Nếu project/codebase context thiếu, stale, contradictory, hoặc low-confidence, dùng workflow skill phù hợp như `project-sync` hoặc `codebase-sync`.

Không đọc toàn bộ project/codebase cache theo mặc định.

## Harness Lifecycle

Trước khi tạo run, classify request:

```txt
User request
  -> Normal Run nếu bounded và verify được trong một run
  -> Epic nếu broad, multi-phase, multi-module, long-running, hoặc không verify sạch trong một run
````

Canonical run artifacts:

```txt
run.yaml
run-manifest.md
00-input.md
01-planner-brief.md
02-implementation-contract.md
03-contract-review.md
04-implementation-report.md
05-evaluator-report.md
06-final-summary.md
```

Normal runs live under:

```txt
.harness/runs/RUN-YYYYMMDD-NNN-task-slug/
```

Epic containers live under:

```txt
.harness/runs/EPIC-YYYYMMDD-NNN-task-slug/
```

Child runs live under:

```txt
.harness/runs/EPIC-YYYYMMDD-NNN-task-slug/runs/RUN-NNN-child-task-slug/
```

Luôn update:

```txt
.harness/runs/RUN_INDEX.md
```

Detailed classification and Epic rules live in:

```txt
.harness/guides/RUN_CLASSIFICATION.md
.harness/guides/LONG_TASK_POLICY.md
```

## Template-Based Subagent Orchestration

Harness core lifecycle roles MUST be executed by separate spawned subagents from fixed templates.

Core lifecycle roles:

1. `planner`
2. `contract_reviewer`
3. `generator`
4. `evaluator`

Required templates:

```txt
.harness/subagents/planner.md
.harness/subagents/contract-reviewer.md
.harness/subagents/generator.md
.harness/subagents/evaluator.md
```

The coordinator/orchestrator is orchestration-only.

The coordinator MUST NOT:

* perform Planner, Contract Reviewer, Generator, or Evaluator work directly;
* create free-form prompts for core lifecycle roles;
* modify role responsibilities, forbidden actions, required artifacts, output schema, evidence requirements, pass/fail criteria, or independence requirements;
* write lifecycle role artifacts on behalf of subagents;
* implement, debug, repair, review, verify, approve, or test application work directly;
* modify application source files, tests, production configuration, generated production artifacts, or project implementation files;
* emulate multiple production roles in one session;
* create `HANDOFF.md` for normal lifecycle transitions;
* continue in degraded single-session fallback.

There is no degraded single-session fallback.

If required template subagents cannot be spawned, block the run before lifecycle role execution.

## Dispatch Semantics

The coordinator dispatches roles with:

```bash
bash .harness/scripts/dispatch-role.sh .harness/runs/<RUN_ID> <role>
```

`dispatch-role.sh` only creates:

```txt
.harness/runs/<RUN_ID>/dispatch/<role>.dispatch.md
```

It does not spawn, execute, or emulate a subagent.

The runtime executor MUST consume the dispatch artifact and spawn the role-specific subagent from the fixed template.

If no real runtime adapter exists, Harness lifecycle execution is blocked unless the current agent/runtime has native subagent spawning and can consume dispatch artifacts.

Before Planner dispatch, runtime availability must be marked:

```bash
bash .harness/scripts/mark-subagent-runtime.sh .harness/runs/<RUN_ID> true
```

If subagent runtime is unavailable:

```bash
bash .harness/scripts/mark-subagent-runtime.sh .harness/runs/<RUN_ID> false "Subagent runtime unavailable"
```

Required blocked message:

```text
Subagent runtime unavailable.
Harness lifecycle requires template-based subagent orchestration.
This run is blocked.
No lifecycle role may be executed in this session.
```

## Coordinator Write Scope

For coordinator/orchestrator sessions, run the write-scope validator before accepting changed files:

```bash
HARNESS_EXECUTOR_ROLE=coordinator \
HARNESS_RUN_DIR=".harness/runs/<RUN_ID>" \
bash .harness/scripts/validate-coordinator-write-scope.sh
```

If it fails, stop with:

```text
BLOCKED_COORDINATOR_WRITE_SCOPE_VIOLATION
```

Coordinator may write only narrow orchestration metadata allowed by the validator.

Coordinator must route source/test/config/implementation changes to Generator.

## Role Boundaries

### Planner

Planner may inspect relevant project context and source/test files for planning.

Planner writes:

```txt
01-planner-brief.md
02-implementation-contract.md
```

Planner must not implement code, approve its own contract, or evaluate final output.

### Contract Reviewer

Contract Reviewer reviews `01-planner-brief.md` and `02-implementation-contract.md`.

Contract Reviewer writes:

```txt
03-contract-review.md
```

The final line of `03-contract-review.md` MUST be exactly one of:

```txt
- Status: approved
- Status: rejected_requires_revision
```

Contract Reviewer must not implement code, silently rewrite the contract, or evaluate final implementation.

### Generator

Generator implements only after contract approval.

Generator writes:

```txt
04-implementation-report.md
```

Generator may edit application source, tests, docs, and config only within the approved contract.

Before editing implementation files, Generator must:

* read target files first;
* inspect nearby code;
* search usages before changing existing functions, classes, routes, commands, or exported APIs;
* avoid unrelated refactors;
* keep changes inside the approved contract.

Generator must not approve or evaluate its own work.

### Evaluator

Evaluator independently evaluates implementation against:

* original user request;
* `01-planner-brief.md`;
* approved `02-implementation-contract.md`;
* `03-contract-review.md`;
* `04-implementation-report.md`;
* code diff;
* command output;
* runtime/browser/API evidence where relevant;
* acceptance criteria.

Evaluator writes:

```txt
05-evaluator-report.md
```

The final line of `05-evaluator-report.md` MUST be exactly one of:

```txt
- Status: pass
- Status: fail
- Status: blocked_insufficient_evidence
```

Evaluator must not patch implementation, weaken acceptance criteria, or approve based only on code inspection or Generator summary.

## Rework Routing

If Evaluator returns:

```txt
- Status: fail
```

or:

```txt
- Status: blocked_insufficient_evidence
```

the coordinator MUST NOT fix implementation directly.

The coordinator may read only the evaluator decision summary, create a bounded Generator rework packet from:

```txt
.harness/templates/generator-rework-packet.template.md
```

Then coordinator must dispatch Generator via `dispatch-role.sh`.

The runtime executor must spawn Generator from the dispatch artifact.

After Generator updates `04-implementation-report.md`, coordinator dispatches Evaluator again.

If Generator cannot be spawned for required implementation or rework, stop with:

```text
BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
```

## Verification

Run verification through the appropriate role.

Default verification guidance lives in:

```txt
.harness/guides/TESTING_POLICY.md
```

Common commands, when relevant:

```bash
bash .harness/scripts/verify.sh
bash .harness/scripts/smoke.sh
```

For Vite apps, when relevant:

```bash
APP_URL=http://localhost:5173 bash .harness/scripts/smoke.sh
```

For UI tasks, build success, static checks, or curl smoke are insufficient. Evaluator must provide behaviour-level evidence for required UI behaviours.

## Parallel Work

If the user gives multiple unrelated tasks, create one run per task.

Before implementation, check active runs for possible file conflicts using the relevant guide/script.

Parallel work policy lives in:

```txt
.harness/guides/PARALLEL_WORK.md
```

Do not overwrite user work or another run’s work. If conflict can cause data loss or ambiguous ownership, stop and surface the tradeoff.

## Legacy Artifacts Are Forbidden

Do not create or rely on:

```txt
HANDOFF.md
03-evaluator-contract-review.md
04-generator-worklog.md
06-fix-report.md
07-final-summary.md
```

Current canonical artifacts are `00` through `06-final-summary.md`.

## Guide Routing

Only load guides relevant to the current task.

Available guide families may include:

```txt
.harness/guides/HARNESS_PRINCIPLES.md
.harness/guides/AGENT_WORKFLOW.md
.harness/guides/PROJECT_DISCOVERY.md
.harness/guides/LANGUAGE_POLICY.md
.harness/guides/RUN_CLASSIFICATION.md
.harness/guides/PLANNING_AND_CONTRACTS.md
.harness/guides/RUNTIME_ROLE_SEPARATION.md
.harness/guides/TESTING_POLICY.md
.harness/guides/PARALLEL_WORK.md
.harness/guides/BACKLOG_POLICY.md
.harness/guides/LONG_TASK_POLICY.md
.harness/guides/LIFECYCLE_ORCHESTRATION.md
.harness/guides/SUBAGENT_EXECUTION.md
```

Do not load the entire guide set by default.

## Priority

When instructions conflict, follow this order:

1. Current user request.
2. Root `AGENTS.md`.
3. Relevant `.harness/project/*`.
4. Relevant `.harness/codebase/*`.
5. `run.yaml` and current run artifacts.
6. Relevant `.harness/guides/*`.
7. Relevant `.harness/workflows/*`.
8. Relevant `.harness/subagents/*`.
9. Templates/scripts in `.harness/`.
10. Agent defaults or assumptions.