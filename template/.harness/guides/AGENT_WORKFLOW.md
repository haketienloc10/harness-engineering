# Agent Workflow

## Luồng mặc định

1. Đọc user request và project adapter liên quan trong `.harness/project/*`.
2. Tạo run bằng `bash .harness/scripts/new-run.sh <task-slug>`.
3. Viết `00-input.md` và `01-planner-brief.md`.
4. Viết `02-implementation-contract.md`.
5. Evaluator review contract trong `03-evaluator-contract-review.md`.
6. Chỉ implement sau khi contract được approve.
7. Ghi worklog trong `04-generator-worklog.md`.
8. Evaluator chạy verification thật và ghi `05-evaluator-report.md`.
9. Nếu fail, viết `06-fix-report.md`, fix trong phạm vi contract, rồi evaluate lại.
10. Kết thúc bằng `07-final-summary.md` và update `RUN_INDEX.md`.

## Kỷ luật phạm vi

Không refactor ngoài yêu cầu. Không sửa Harness guides/templates/scripts trừ khi task yêu cầu. Nếu phát hiện cải tiến Harness reusable, ghi proposal vào `.harness/backlog/HARNESS_BACKLOG.md`.
