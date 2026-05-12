# Agent Workflow

## Luồng mặc định

Default workflow là multi-agent / multi-session. Production-grade work yêu cầu handoff giữa các runtime session/agent riêng:

```txt
User Request
  -> Classify Request
  -> Epic or Normal Run
  -> Planner Agent
  -> Contract Reviewer Agent
  -> Generator Agent
  -> Evaluator Agent
  -> Final Summary
```

Single-agent simulation chỉ được dùng như degraded fallback cho local experimentation, low-risk documentation-only tasks, learning/demo workflows, hoặc task được user đánh dấu rõ là fallback-allowed. Fallback bị cấm cho production implementation, Epic, child runs, UI/API behaviour implementation, và task cần independent review/evaluation.

## Bootstrap Run

1. Đọc user request.
2. Đọc project adapter nếu relevant.
3. Classify task là Normal Run hay Epic bằng `.harness/guides/RUN_CLASSIFICATION.md`.
4. Nếu Epic required, tạo Epic và child-run plan. Không tạo normal run cho task broad/multi-phase.
5. Chỉ tạo normal run nếu bounded và verify được như một đơn vị.
6. Tạo normal run bằng `bash .harness/scripts/new-run.sh <task-slug>` hoặc child run bằng `bash .harness/scripts/new-run.sh --within <EPIC-ID> <task-slug>`.
7. Enforce lifecycle state và role separation bằng `.harness/guides/LIFECYCLE_ORCHESTRATION.md`; nếu có Codex subagents, dùng `.harness/guides/SUBAGENT_EXECUTION.md`.

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

Planner Agent phải kết thúc bằng handoff note cho Contract Reviewer Agent.
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

- `03-evaluator-contract-review.md`.

Decision:

- `APPROVED`: Generator may start.
- `REJECTED`: Planner must revise contract before implementation.

Contract Reviewer Agent phải là runtime session khác với Planner Agent trong production mode.
Contract Reviewer Agent phải dừng sau `03-evaluator-contract-review.md`.

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
- `04-generator-worklog.md`;
- `06-fix-report.md` if applicable.

Generator Agent chỉ implement sau khi `03-evaluator-contract-review.md` có `Status: APPROVED`.
Generator Agent phải dừng sau `04-generator-worklog.md` hoặc `06-fix-report.md`; không tự evaluate.

## Evaluator Phase

Inputs:

- original input;
- planner brief;
- approved contract;
- contract review;
- generator worklog;
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
- `07-final-summary.md` after verified completion.

Evaluator Agent phải là runtime session khác với Generator Agent trong production mode. Evaluation phải dựa trên visible artifacts, diff, command output, runtime evidence, browser/API evidence, logs, và acceptance criteria.
Evaluator Agent phải dừng sau `05-evaluator-report.md` và `07-final-summary.md` nếu final summary được giao cho Evaluator.

## Handoff Note Format

Mỗi role phải kết thúc bằng:

```md
## Handoff

- Completed role:
- Artifacts produced:
- Next required role:
- Allowed next actions:
- Blocked actions:
- Notes for next role:
```

Nếu không có independent session cho role tiếp theo trong production implementation, current agent phải dừng ở boundary và ghi `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`.

## Kỷ luật phạm vi

Không refactor ngoài yêu cầu. Không sửa Harness guides/templates/scripts trừ khi task yêu cầu. Nếu phát hiện cải tiến Harness reusable, ghi proposal vào `.harness/backlog/HARNESS_BACKLOG.md`.
