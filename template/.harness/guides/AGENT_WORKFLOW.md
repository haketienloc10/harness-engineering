# Agent Workflow

## Luồng mặc định

1. Đọc user request và project adapter liên quan trong `.harness/project/*`.
2. Trước khi tạo run, quyết định task có thuộc Epic không. Với long task, đọc `.harness/guides/LONG_TASK_POLICY.md` và tạo hoặc dùng Epic active.
3. Tạo run bằng `bash .harness/scripts/new-run.sh <task-slug>` hoặc `bash .harness/scripts/new-run.sh <task-slug> --epic <EPIC-ID>`.
4. Viết `00-input.md` và `01-planner-brief.md`.
5. Viết `02-implementation-contract.md`.
6. Evaluator review contract trong `03-evaluator-contract-review.md`.
7. Chỉ implement sau khi contract được approve.
8. Ghi worklog trong `04-generator-worklog.md`.
9. Evaluator chạy verification thật và ghi `05-evaluator-report.md`.
10. Nếu fail, viết `06-fix-report.md`, fix trong phạm vi contract, rồi evaluate lại.
11. Kết thúc bằng `07-final-summary.md` và update `RUN_INDEX.md`.

## Kỷ luật phạm vi

Không refactor ngoài yêu cầu. Không sửa Harness guides/templates/scripts trừ khi task yêu cầu. Nếu phát hiện cải tiến Harness reusable, ghi proposal vào `.harness/backlog/HARNESS_BACKLOG.md`.
