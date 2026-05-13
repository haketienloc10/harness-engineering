# Agent Workflow

## Luồng mặc định

Default workflow là strict template-based subagent orchestration:

```txt
User Request
  -> Classify Request
  -> Epic or Normal Run
  -> Planner subagent from `.harness/subagents/planner.md`
  -> Contract Reviewer subagent from `.harness/subagents/contract-reviewer.md`
  -> Generator subagent from `.harness/subagents/generator.md`
  -> Evaluator subagent from `.harness/subagents/evaluator.md`
  -> Final Summary
```

Không có degraded single-session fallback.

If subagent spawning is unavailable, block the run before Planner execution.

When a phase requires another role, call `.harness/scripts/dispatch-role.sh` and spawn the corresponding role subagent from its fixed template.

Coordinator chỉ được điều phối. Coordinator không được implement, debug, repair, review, verify, approve, edit source/tests/config, hoặc viết lifecycle role artifact thay subagent.

Do not create `HANDOFF.md`.

Each role artifact may include a short "Next role" note for traceability, but this note is not a handoff file and must not replace subagent spawning.

## Bootstrap Run

1. Đọc user request.
2. Đọc project adapter nếu relevant.
3. Classify task là Normal Run hay Epic bằng `.harness/guides/RUN_CLASSIFICATION.md`.
4. Nếu Epic required, tạo Epic và child-run plan. Không tạo normal run cho task broad/multi-phase.
5. Chỉ tạo normal run nếu bounded và verify được như một đơn vị.
6. Tạo normal run bằng `bash .harness/scripts/new-run.sh <task-slug>` hoặc child run bằng `bash .harness/scripts/new-run.sh --within <EPIC-ID> <task-slug>`.
7. Enforce lifecycle state và role separation bằng `.harness/guides/LIFECYCLE_ORCHESTRATION.md` và `.harness/guides/SUBAGENT_EXECUTION.md`.
8. Nếu runtime không thể spawn subagents, update `run-manifest.md`, set `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, và dừng.

## Planner Phase

Inputs:

- user request;
- project adapter files;
- relevant guides;
- source/test inspection as needed for planning.

Forbidden:

- application code changes;
- contract approval;
- evaluation approval.

Outputs:

- `00-input.md`;
- `01-planner-brief.md`;
- `02-implementation-contract.md`.

Planner Agent có thể ghi "Next role" note cho Contract Reviewer Agent để traceability.
Planner Agent phải dừng sau `02-implementation-contract.md`; không implement và không tự approve contract.

## Contract Review Phase

Inputs:

- `00-input.md`;
- `01-planner-brief.md`;
- `02-implementation-contract.md`;
- project rules/verification notes if needed.

Forbidden:

- application code changes;
- hidden assumptions from Planner;
- approving vague or untestable contracts;
- silently rewriting the contract.

Outputs:

- `03-contract-review.md`.

Decision:

- `approved`: Generator may start.
- `rejected_requires_revision`: Planner must revise contract before implementation.

Contract Reviewer must be a spawned subagent from `.harness/subagents/contract-reviewer.md`.
Contract Reviewer Agent phải dừng sau `03-contract-review.md`.

## Generator Phase

Inputs:

- approved contract;
- contract review;
- relevant source code/tests.

Forbidden:

- changing scope beyond contract;
- approving its own changes;
- weakening verification criteria.

Outputs:

- code changes;
- `04-implementation-report.md`.

Generator Agent chỉ implement sau khi `03-contract-review.md` có `Status: approved`.
Generator Agent phải dừng sau `04-implementation-report.md`; không tự evaluate.

## Rework Routing Phase

Khi `05-evaluator-report.md` trả `FAIL`, `REJECTED`, `NEEDS_FIX`, `blocked_insufficient_evidence`, hoặc kết quả không pass tương đương:

1. Coordinator chỉ đọc evaluator decision summary.
2. Coordinator tạo bounded Generator rework packet từ `.harness/templates/generator-rework-packet.template.md`.
3. Coordinator spawn Generator từ `.harness/subagents/generator.md`.
4. Generator sửa implementation và cập nhật `04-implementation-report.md`.
5. Coordinator spawn Evaluator từ `.harness/subagents/evaluator.md` lại.

Coordinator không được đọc source để tự repair, không được edit source/tests/config, không được thêm test trực tiếp, không được chạy fix loop, và không được viết role artifact thay Generator.

Nếu không spawn được Generator, stop với:

```text
BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
```

## Evaluator Phase

Inputs:

- original input;
- planner brief;
- approved contract;
- contract review;
- implementation report;
- git diff;
- verification commands and outputs;
- runtime/UI/API evidence when relevant.

Forbidden:

- approving by code inspection only;
- trusting Generator statements without evidence;
- expanding scope silently;
- ignoring failed tests.

Outputs:

- `05-evaluator-report.md`;
- final pass/fail decision;
- `06-final-summary.md` after verified completion.

Evaluator must be a spawned subagent from `.harness/subagents/evaluator.md` and must be separate from Generator. Evaluation phải dựa trên visible artifacts, diff, command output, runtime evidence, browser/API evidence, logs, và acceptance criteria.
Evaluator Agent phải dừng sau `05-evaluator-report.md` và `06-final-summary.md` nếu final summary được giao cho Evaluator.

## Next Role Note

Each role artifact may include a short Next role note for traceability.

This note is not the same as `HANDOFF.md`.

Do not create `HANDOFF.md`.

The note must not replace subagent spawning.

Mỗi role có thể kết thúc bằng:

```md
## Next Role

- Completed role:
- Artifacts produced:
- Next required role:
- Allowed next actions:
- Blocked actions:
- Notes for next role:
```

Nếu không có subagent runtime cho role tiếp theo, current agent phải block run, update `run-manifest.md`, ghi `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, và dừng.

## Kỷ luật phạm vi

Không refactor ngoài yêu cầu. Không sửa Harness guides/templates/scripts trừ khi task yêu cầu. Nếu phát hiện cải tiến Harness reusable, ghi proposal vào `.harness/backlog/HARNESS_BACKLOG.md`.
