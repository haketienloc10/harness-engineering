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

## Priority order

Khi instruction xung đột, theo thứ tự:

1. Current user request.
2. Root `AGENTS.md`.
3. Project adapter files trong `.harness/project/*`.
4. Relevant files trong `.harness/guides/*`.
5. Templates trong `.harness/templates/*`.
6. Agent defaults hoặc assumptions.

Generated Harness artifacts nên dùng ngôn ngữ người dùng đang dùng, trừ technical identifier, command, path, code, config key, log, error message, API field, schema key, package name, hoặc copied tool output.

---

## Mandatory Harness lifecycle

Với mọi implementation task không tầm thường, tạo một run dưới:

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
User Request
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

Create an Epic only when the task has multiple milestones, multiple user flows, multiple modules, uncertain scope, or cannot be verified cleanly in one run, and the planner can identify at least two independently verifiable child runs before implementation.

If only one concrete run is known, create a normal run and add follow-up proposal/backlog instead of creating an Epic.

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

Relevant guide:

```txt
.harness/guides/LONG_TASK_POLICY.md
```

---

## Role separation

Production workflow yêu cầu các agent/session riêng cho:

- Planner Agent
- Contract Reviewer Agent
- Generator Agent
- Evaluator Agent

Một role session không được approve output trước đó của chính nó. Planner Agent tạo `00-input.md`, `01-planner-brief.md`, và `02-implementation-contract.md`, nhưng không được implement application code hoặc approve contract của mình. Contract Reviewer Agent review contract và phải approve/reject trong `03-evaluator-contract-review.md`, nhưng không được sửa application code hoặc rewrite contract âm thầm. Generator Agent chỉ implement sau khi contract được approve và không được tự approve implementation. Evaluator Agent phải là session/agent khác với Generator trong production mode.

Evaluator không được rely vào hidden reasoning hoặc memory từ Planner/Generator. Evaluation chỉ được dựa trên visible artifacts, code diff, command output, runtime evidence, browser evidence, API evidence, logs, và acceptance criteria. Evaluator không được approve chỉ bằng code inspection.

Single-agent simulation chỉ là degraded fallback cho local experimentation, task nhỏ low-risk, hoặc môi trường không hỗ trợ multi-agent. Fallback artifact phải ghi rõ:

```yaml
runtime_mode: fallback_single_session
independence: degraded
reason: "<why separate sessions were unavailable>"
```

Fallback mode không phải production-grade.

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
.harness/guides/PLANNING_AND_CONTRACTS.md
.harness/guides/RUNTIME_ROLE_SEPARATION.md
.harness/guides/TESTING_POLICY.md
.harness/guides/PARALLEL_WORK.md
.harness/guides/BACKLOG_POLICY.md
.harness/guides/LONG_TASK_POLICY.md
```

Không load toàn bộ guides theo mặc định.
