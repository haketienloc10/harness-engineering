# Agent Workflow

## Luồng mặc định

Default workflow là multi-agent / multi-session. Production-grade work yêu cầu handoff giữa các runtime session/agent riêng:

```txt
User Request
  -> Planner Agent
  -> Contract Reviewer Agent
  -> Generator Agent
  -> Evaluator Agent
  -> Final Summary
```

Single-agent simulation chỉ được dùng như degraded fallback cho local experimentation, task nhỏ low-risk, hoặc môi trường không hỗ trợ multi-agent. Fallback artifacts phải ghi `runtime_mode: fallback_single_session` và `independence: degraded`.

## Bootstrap Run

1. Đọc user request và project adapter liên quan trong `.harness/project/*`.
2. Trước khi tạo run, quyết định task có thuộc Epic container không. Với long task, đọc `.harness/guides/LONG_TASK_POLICY.md`; chỉ tạo Epic nếu xác định được ít nhất hai child runs có thể verify độc lập.
3. Tạo normal run bằng `bash .harness/scripts/new-run.sh <task-slug>` hoặc child run bằng `bash .harness/scripts/new-run.sh --within <EPIC-ID> <task-slug>`.
4. Dùng `.harness/guides/RUNTIME_ROLE_SEPARATION.md` để chọn role/session tiếp theo và ghi runtime metadata vào artifact.

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

## Kỷ luật phạm vi

Không refactor ngoài yêu cầu. Không sửa Harness guides/templates/scripts trừ khi task yêu cầu. Nếu phát hiện cải tiến Harness reusable, ghi proposal vào `.harness/backlog/HARNESS_BACKLOG.md`.
