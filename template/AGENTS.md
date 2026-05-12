# AGENTS.md

## Mục đích

Repository này đã cài **Harness** để làm việc với AI theo cách có kế hoạch, có contract, có log, và có bằng chứng kiểm chứng.

File này là bootstrap instruction của target repository. Giữ file này ngắn; quy trình chi tiết nằm trong `.harness/guides/*` và chỉ đọc khi liên quan đến task hiện tại.

Người dùng vẫn có thể đưa một yêu cầu phát triển bình thường. Production workflow yêu cầu các runtime session/agent riêng cho Planner, Contract Reviewer, Generator, và Evaluator, được điều phối qua artifact trong `.harness/runs/*`.

---

## Harness Skill Discovery

This project uses project-local Harness Workflow Skills.

Before non-trivial work, read:

```txt
.harness/HARNESS_SKILLS.md
```

Use it as the skill registry.

Select the relevant skill by name, description, and trigger conditions.

Load only the selected skill file.

Do not load every skill file by default.

---

## Project Context Requirement

Before any run or epic, check whether `.harness/project/*` exists and appears current.

If project context is missing, stale, contradictory, or low-confidence, use the `project-sync` Harness workflow skill from `.harness/HARNESS_SKILLS.md`.

The user may request this skill at any time to refresh project context.

---

## Ranh giới repository

`.harness/` là workflow infrastructure của target repository. Nó chứa guides, templates, scripts, run records, project adapter files, và backlog cho AI-assisted development.

`.harness/` không phải application source tree. Khi làm task, agent phải inspect source code, tests, runtime behaviour, và architecture thực tế của target repository bên ngoài `.harness/`.

---

## Project adapter

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

---

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

---

## Priority order

Khi instruction xung đột, theo thứ tự:

1. Current user request.
2. Root `AGENTS.md`.
3. Project adapter files trong `.harness/project/*`.
4. Source-navigation files liên quan trong `.harness/codebase/*`.
5. Relevant files trong `.harness/guides/*`.
6. Templates trong `.harness/templates/*`.
7. Agent defaults hoặc assumptions.

Generated Harness artifacts nên dùng ngôn ngữ người dùng đang dùng, trừ technical identifier, command, path, code, config key, log, error message, API field, schema key, package name, hoặc copied tool output.

---

## Mandatory Harness lifecycle

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

Một run hợp lệ phải có và duy trì:

```txt
run.yaml
00-input.md
01-planner-brief.md
02-implementation-contract.md
03-evaluator-contract-review.md
04-generator-worklog.md
05-evaluator-report.md
07-final-summary.md
```

Nếu evaluation fail và cần fix, viết thêm:

```txt
06-fix-report.md
```

Luôn update:

```txt
.harness/runs/RUN_INDEX.md
```

Không sửa application code trước khi `03-evaluator-contract-review.md` approve `02-implementation-contract.md`.

Lifecycle chuẩn:

```txt
Classify Request
  -> Epic or Normal Run
  -> Planner Agent
  -> Contract Reviewer Agent
  -> Generator Agent
  -> Evaluator Agent
  -> Final Summary
```

---

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

Each Epic must be decomposed into smaller child runs. Each child run keeps the normal Planner Agent -> Contract Reviewer Agent -> Generator Agent -> Evaluator Agent -> Final Summary lifecycle.

Before creating a run for a long task, check whether an active Epic should own the run.

Relevant guides:

```txt
.harness/guides/RUN_CLASSIFICATION.md
.harness/guides/LONG_TASK_POLICY.md
```

---

## Production Role Separation

Production workflow yêu cầu các agent/session riêng cho:

- Planner Agent
- Contract Reviewer Agent
- Generator Agent
- Evaluator Agent

Một role session không được approve output trước đó của chính nó. Planner Agent tạo `00-input.md`, `01-planner-brief.md`, và `02-implementation-contract.md`, nhưng không được implement application code hoặc approve contract của mình. Contract Reviewer Agent review contract và phải approve/reject trong `03-evaluator-contract-review.md`, nhưng không được sửa application code hoặc rewrite contract âm thầm. Generator Agent chỉ implement sau khi contract được approve và không được tự approve implementation. Evaluator Agent phải là session/agent khác với Generator trong production mode.

Evaluator không được rely vào hidden reasoning hoặc memory từ Planner/Generator. Evaluation chỉ được dựa trên visible artifacts, code diff, command output, runtime evidence, browser evidence, API evidence, logs, và acceptance criteria. Evaluator không được approve chỉ bằng code inspection.

For production-grade implementation work, Planner, Contract Reviewer, Generator, and Evaluator must run as separate runtime sessions.

A single runtime session must not approve its own contract or evaluate its own implementation.

If independent sessions are unavailable, stop at the role boundary and produce a handoff prompt for the next role.

Single-agent simulation chỉ là degraded fallback cho local experimentation, low-risk documentation-only tasks, learning/demo workflows, hoặc task được user đánh dấu rõ là fallback-allowed. Fallback artifact phải ghi rõ:

```yaml
runtime_mode: fallback_single_session
independence: degraded
reason: "<why fallback is allowed for this task>"
```

Fallback mode không phải production-grade.

Fallback single-session bị cấm cho:

- Epic;
- child runs inside Epic;
- broad implementation tasks;
- production implementation;
- UI/API behaviour implementation;
- any task where user explicitly requires independent roles.

Nếu production implementation cần session độc lập mà environment không tạo được tự động, current agent phải dừng ở role boundary và tạo handoff prompt cho independent session tiếp theo.

Chi tiết:

```txt
.harness/guides/RUNTIME_ROLE_SEPARATION.md
```

---

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

---

## Code change rules

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

---

## Parallel work

Nếu user đưa nhiều task không liên quan, tạo một run cho mỗi task.

Trước implementation, kiểm tra active runs để phát hiện file conflicts. Nếu các run có thể modify cùng file, ghi conflict và ưu tiên separate branch hoặc worktree.

---

## Khi cần đọc guides

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
```

Không load toàn bộ guides theo mặc định.
