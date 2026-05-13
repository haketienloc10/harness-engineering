# AGENTS.md

Repository này đã cài **Harness** để điều phối AI-assisted development bằng artifact, role policy, và lifecycle state.

File này là bootstrap instruction của target repository. Quy trình chi tiết nằm trong `.harness/guides/*`; chỉ đọc guide liên quan đến task hiện tại.

## Bootstrap Rules

- Reply theo ngôn ngữ người dùng; giữ code, command, path, API name, logs, và identifiers ở dạng gốc.
- Trước non-trivial work, đọc `.harness/HARNESS_SKILLS.md` và chỉ load skill liên quan.
- Trước lifecycle execution, đọc `.harness/guides/LIFECYCLE_ORCHESTRATION.md`.
- Trước lifecycle execution, đọc `.harness/guides/SUBAGENT_EXECUTION.md` và `.harness/workflows/default-lifecycle.md`.
- Dùng `run.yaml` làm authoritative workflow state.
- Không sửa application code trước khi Contract Reviewer approve contract và `run.yaml` cho phép Generator.
- Evaluator phải độc lập với Generator và phải có evidence thật.
- Không kết thúc bằng generic “Suggested Next Steps” khi bước tiếp theo có thể được biểu diễn bằng executor dispatch.

## Harness Skill Discovery

This project uses project-local Harness Workflow Skills.

Before non-trivial work, read:

```txt
.harness/HARNESS_SKILLS.md
```

Use it as the skill registry. Select the relevant skill by name, description, and trigger conditions. Load only the selected skill file. Do not load every skill file by default.

## Project Context Requirement

Before any run or epic, check whether `.harness/project/*` exists and appears current.

If project context is missing, stale, contradictory, or low-confidence, use the `project-sync` Harness workflow skill from `.harness/HARNESS_SKILLS.md`.

The user may request this skill at any time to refresh project context.

## Repository Boundary

`.harness/` là workflow infrastructure của target repository. Nó chứa guides, templates, scripts, run records, project adapter files, codebase cache, và backlog cho AI-assisted development.

`.harness/` không phải application source tree. Khi làm task, inspect source code, tests, runtime behaviour, và architecture thực tế của target repository bên ngoài `.harness/`.

Harness là seed workflow layer, không phải application source tree.

## Project Adapter

Trước khi lập kế hoạch cho task implementation không tầm thường, đọc các file này nếu có:

```txt
.harness/project/PROJECT_PROFILE.md
.harness/project/PROJECT_CONTEXT.md
.harness/project/PROJECT_RULES.md
.harness/project/PROJECT_VERIFICATION.md
.harness/project/PROJECT_ARCHITECTURE.md
.harness/project/PROJECT_GLOSSARY.md
.harness/project/PROJECT_OPEN_QUESTIONS.md
```

Nếu các file này thiếu hoặc có vẻ cũ, đọc `.harness/HARNESS_SKILLS.md` và chạy Harness workflow skill `project-sync`.

Project context chỉ đáng tin khi có evidence hiện tại. Ưu tiên manual notes và quyết định local của target repository khi có xung đột, nhưng ghi rõ uncertainty nếu evidence chưa đủ.

## Codebase Knowledge Base

`.harness/codebase/*` là source-navigation và change-impact cache thuộc target repository. Nó không thay thế `.harness/project/*` và không được duplicate project-level facts.

Trước coding run không tầm thường:

1. Đọc `.harness/project/*` để lấy project-level context.
2. Đọc `.harness/codebase/CODEBASE_INDEX.md` nếu có.
3. Chỉ đọc các `.harness/codebase/*` docs liên quan đến task.
4. Dùng codebase docs để xác định source files, modules, technical flows, và impact areas cần inspect.
5. Sau đó inspect actual source files trước khi edit.
6. Search usages trước khi đổi existing functions/classes/routes/commands/exported APIs.
7. Nếu source evidence mâu thuẫn codebase docs, update `.harness/codebase/*` hoặc mark stale context.

Nếu `.harness/codebase/*` thiếu, stale, contradictory, hoặc low-confidence, đọc `.harness/HARNESS_SKILLS.md` và chạy Harness workflow skill `codebase-sync`.

## Mandatory Harness Lifecycle

Trước khi tạo bất kỳ run nào, classify request:

```txt
User request
  -> classify as Normal Run or Epic
  -> create Epic if broad/multi-phase
  -> create normal run only if bounded
```

Với mọi implementation task không tầm thường và bounded, tạo một run dưới:

```txt
.harness/runs/RUN-YYYYMMDD-NNN-task-slug/
```

Nếu implementation task là một phần của Epic, tạo child run dưới:

```txt
.harness/runs/EPIC-YYYYMMDD-NNN-task-slug/runs/RUN-NNN-child-task-slug/
```

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

Lifecycle chuẩn:

```txt
Classify Request
  -> Epic or Normal Run
  -> Planner subagent
  -> Contract Reviewer subagent
  -> Generator subagent
  -> Evaluator subagent
  -> Final Summary
```

Luôn update `.harness/runs/RUN_INDEX.md` khi tạo run/epic.

## Long Task / Epic Policy

For long-running tasks, do not create a single giant run.

Epic is a planning and coordination run container, not a standalone metadata folder and not a normal implementation run.

Epic is mandatory when the task has multiple phases, multiple milestones, multiple user flows, multiple modules, uncertain or expanding scope, cannot be verified cleanly in one run, or mentions wording such as `phase`, `phase 1-4`, `part 1-4`, `core loop`, `full feature`, `complete playable`, `end-to-end`, `MVP`, `large task`, or `long task`.

A task named like `phase 1-4` must not become one normal run.

If a task qualifies as an Epic but only one child run is known, Planner must reduce scope to one bounded normal run or ask/derive additional decomposition before implementation. Do not use an oversized normal run as a workaround.

Epic artifacts live under:

```txt
.harness/runs/EPIC-YYYYMMDD-NNN-task-slug/
```

Child runs live under:

```txt
.harness/runs/EPIC-YYYYMMDD-NNN-task-slug/runs/RUN-NNN-child-task-slug/
```

Create child runs with:

```bash
bash .harness/scripts/new-run.sh --within EPIC-YYYYMMDD-NNN-task-slug "child task"
```

Each Epic must be decomposed into smaller child runs. Each child run keeps the normal Planner -> Contract Reviewer -> Generator -> Evaluator -> Final Summary lifecycle through template subagents.

Before creating a run for a long task, check whether an active Epic should own the run.

Relevant guides:

```txt
.harness/guides/RUN_CLASSIFICATION.md
.harness/guides/LONG_TASK_POLICY.md
```

## Template-Based Subagent Orchestration

Harness core lifecycle roles MUST be executed by separate spawned subagents.

The coordinator MUST call `.harness/scripts/dispatch-role.sh` to create a dispatch artifact for each core role from its predefined template under `.harness/subagents/`.

Core lifecycle roles are:

1. Planner
2. Contract Reviewer
3. Generator
4. Evaluator

The coordinator MUST NOT create free-form prompts for these roles. `dispatch-role.sh` does not accept free-form role prompts.

The coordinator MUST NOT execute these roles itself.

The coordinator is orchestration-only. It MUST NOT perform implementation, review, verification, debugging, or repair work directly.

The coordinator MAY make task-specific artifacts available to the selected role template, including:

- original user request
- project context summary
- relevant files
- previous role artifacts
- acceptance criteria
- verification commands
- constraints

The coordinator MUST NOT modify:

- role responsibilities
- forbidden actions
- required artifacts
- output schema
- evidence requirements
- pass/fail criteria
- independence requirements

The coordinator MUST NOT modify application source files, tests, production configuration, generated production artifacts, or project implementation files. Any source/test/config change MUST be performed by the Generator role.

For coordinator/orchestrator sessions, run the write-scope validator before accepting changed files:

```bash
HARNESS_EXECUTOR_ROLE=coordinator \
HARNESS_RUN_DIR=".harness/runs/<RUN_ID>" \
bash .harness/scripts/validate-coordinator-write-scope.sh
```

The validator enforces the narrow orchestration metadata allowlist documented in `.harness/guides/SUBAGENT_EXECUTION.md#coordinator-write-scope-validator`. If it fails, stop with `BLOCKED_COORDINATOR_WRITE_SCOPE_VIOLATION` and route source/test/config work to Generator.

If subagent spawning is unavailable, the run MUST be blocked.

There is no degraded single-session fallback.

## No Subagent Runtime, No Harness Run

Harness lifecycle execution requires real subagent spawning.

If the current runtime cannot spawn subagents, the coordinator MUST block the run before Planner execution.

The coordinator MUST NOT emulate subagents by writing multiple role artifacts in one session.

The coordinator MUST NOT continue in degraded mode.

A blocked run is valid and preferable to a fake multi-role run.

Required blocking message:

```text
Subagent runtime unavailable.
Harness lifecycle requires template-based subagent orchestration.
This run is blocked.
No lifecycle role may be executed in this session.
```

## Role Independence Audit

A Harness run is invalid if any of the following is true:

- Planner, Contract Reviewer, Generator, and Evaluator were performed by the same session.
- The coordinator wrote lifecycle artifacts on behalf of role subagents.
- A core role was executed from a free-form prompt instead of a role template.
- The run continued after detecting unavailable subagent runtime.
- Evaluator approved without independent evidence.

## Production Role Separation

Production workflow yêu cầu template subagents riêng cho:

- Planner
- Contract Reviewer
- Generator
- Evaluator

Planner tạo planning artifacts nhưng không được implement application code hoặc approve contract của mình. Contract Reviewer review contract và phải approve/reject trong `03-contract-review.md`, nhưng không được sửa application code hoặc rewrite contract âm thầm. Generator chỉ implement sau khi contract được approve và không được tự approve implementation. Evaluator phải là spawned subagent khác với Generator.

Evaluator không được rely vào hidden reasoning hoặc memory từ Planner/Generator. Evaluation chỉ được dựa trên visible artifacts, code diff, command output, runtime evidence, browser evidence, API evidence, logs, và acceptance criteria. Evaluator không được approve chỉ bằng code inspection.

For production-grade implementation work, Planner, Contract Reviewer, Generator, and Evaluator must run as separate template-based subagents.

A single runtime session must not approve its own contract or evaluate its own implementation.

Fallback execution is forbidden. If required template subagents cannot be spawned, block the run before role execution.

Chi tiết:

```txt
.harness/guides/RUNTIME_ROLE_SEPARATION.md
.harness/guides/SUBAGENT_EXECUTION.md
```

## Role Execution

Harness uses template-based subagent orchestration, not handoff files, for role transitions.

The top-level agent is the coordinator/orchestrator.

The coordinator must run:

```bash
bash .harness/scripts/dispatch-role.sh .harness/runs/<RUN_ID> <role>
```

for the required role whenever the workflow enters a role-owned phase.

`dispatch-role.sh` creates only:

```txt
.harness/runs/<RUN_ID>/dispatch/<role>.dispatch.md
```

It does not spawn, execute, or emulate a subagent.

The runtime executor MUST consume the dispatch artifact and spawn the required role-specific subagent from:

```txt
.harness/subagents/<role>.md
```

Required role templates:

- `.harness/subagents/planner.md`
- `.harness/subagents/contract-reviewer.md`
- `.harness/subagents/generator.md`
- `.harness/subagents/evaluator.md`

If no real runtime adapter exists, Harness lifecycle execution is blocked unless the current agent/runtime has native subagent spawning and can consume `.harness/runs/<RUN_ID>/dispatch/<role>.dispatch.md`.

The coordinator must not create `HANDOFF.md` to move between roles.

The coordinator must not emulate multiple production roles in one agent response.

## Evaluator Failure Routing

When Evaluator returns `FAIL`, `REJECTED`, `NEEDS_FIX`, `blocked_insufficient_evidence`, or any equivalent non-passing result, the coordinator MUST NOT fix the implementation directly.

The coordinator may read only the evaluator decision summary, create a bounded Generator rework packet from `.harness/templates/generator-rework-packet.template.md`, spawn Generator, wait for Generator output, and then spawn Evaluator again.

If the runtime cannot spawn Generator for required implementation or rework, stop with:

```text
BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
```

## Execution Namespace

```txt
.harness/runs/RUN-*                 # normal runs
.harness/runs/EPIC-*                # Epic containers
.harness/runs/EPIC-*/runs/RUN-*     # child runs
```

Không tạo primary Epic data trong `.harness/epics/*`; nếu tồn tại thì chỉ coi là legacy/read-only.

## Verification

Chạy verification thật khi có thể:

```bash
bash .harness/scripts/verify.sh
```

Nếu app có runtime UI hoặc API behaviour, chạy thêm:

```bash
bash .harness/scripts/smoke.sh
```

Với Vite app:

```bash
APP_URL=http://localhost:5173 bash .harness/scripts/smoke.sh
```

Với UI task, build success, static checks, hoặc curl smoke chưa đủ. Evaluator phải có behaviour-level evidence cho từng UI behaviour bắt buộc.

## Code Change Rules

Trước khi edit files:

- đọc target file trước;
- inspect nearby code;
- search usages trước khi đổi existing functions/classes;
- tránh unrelated refactors;
- giữ thay đổi trong approved contract.

Không sửa Harness guides, templates, hoặc scripts trừ khi user yêu cầu. Nếu một run phát hiện cải tiến Harness tái sử dụng được, thêm proposal cụ thể vào:

```txt
.harness/backlog/HARNESS_BACKLOG.md
```

## Parallel Work

Nếu user đưa nhiều task không liên quan, tạo một run cho mỗi task.

Trước implementation, kiểm tra active runs để phát hiện file conflicts:

```bash
bash .harness/scripts/check-conflicts.sh RUN-YYYYMMDD-NNN-task-slug
```

Nếu các run có thể modify cùng file, ghi conflict và ưu tiên sequence work, separate branch, hoặc worktree. Nếu conflict có thể gây overwrite hoặc làm mất thay đổi của user/run khác, dừng và surface tradeoff.

## When To Read Guides

Chỉ load guide liên quan:

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

Không load toàn bộ guides theo mặc định.

## Priority

1. Current user request.
2. Root `AGENTS.md`.
3. `.harness/project/*` và `.harness/codebase/*` liên quan.
4. `run.yaml` và artifact trong current run.
5. Relevant `.harness/guides/*`.
6. Templates/scripts trong `.harness/`.
