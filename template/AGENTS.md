# AGENTS.md

## Mục đích

Repository này đã cài **Harness** để làm việc với AI theo cách có kế hoạch, có contract, có log, và có bằng chứng kiểm chứng.

File này là bootstrap instruction của target repository. Giữ file này ngắn; quy trình chi tiết nằm trong `.harness/guides/*` và chỉ đọc khi liên quan đến task hiện tại.

Người dùng vẫn có thể đưa một yêu cầu phát triển bình thường. Agent phải tự điều phối các vai trò Planner, Generator, và Evaluator thông qua artifact trong `.harness/runs/*`.

---

## Ranh giới repository

`.harness/` là workflow infrastructure của target repository. Nó chứa guides, templates, scripts, run records, project adapter files, và backlog cho AI-assisted development.

`.harness/` không phải application source tree. Khi làm task, agent phải inspect source code, tests, runtime behaviour, và architecture thực tế của target repository bên ngoài `.harness/`.

---

## Project adapter

Trước khi lập kế hoạch cho task implementation không tầm thường, đọc các file này nếu có:

```txt
.harness/project/PROJECT_MAP.md
.harness/project/SOURCE_OF_TRUTH.md
.harness/project/STACK_PROFILE.md
.harness/project/VALIDATION_PROFILE.md
.harness/project/MODULE_MAP.md
.harness/project/LOCAL_DECISIONS.md
```

Nếu các file này thiếu hoặc có vẻ cũ, chạy:

```bash
bash .harness/scripts/inspect-project.sh
```

Discovery output chỉ là observed facts, không phải absolute truth. Ưu tiên manual notes và quyết định local của target repository khi có xung đột.

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

---

## Role separation

Agent có thể đóng Planner, Generator, và Evaluator trong cùng một conversation turn, nhưng artifacts phải tách biệt.

- Planner định nghĩa goal, scope, non-scope, acceptance criteria, impacted areas, risks, và unknowns.
- Generator chỉ implement approved contract.
- Evaluator kiểm chứng bằng original input, planner brief, contract, acceptance criteria, và evidence thực tế.

Evaluator không được approve chỉ bằng code inspection.

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
.harness/guides/TESTING_POLICY.md
.harness/guides/PARALLEL_WORK.md
.harness/guides/BACKLOG_POLICY.md
```

Không load toàn bộ guides theo mặc định.
