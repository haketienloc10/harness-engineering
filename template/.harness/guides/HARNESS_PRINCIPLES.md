# Harness Principles

Harness giúp AI-assisted development có thể audit và verify được.

## Nguyên tắc

- Mỗi task không tầm thường phải có run artifact.
- Phải classify request trước khi tạo run; task lớn hoặc multi-phase phải thành Epic.
- Planner, Contract Reviewer, Generator, và Evaluator phải chạy bằng spawned subagents từ fixed templates.
- Nếu runtime không spawn được subagents, Harness run phải block trước Planner execution.
- Implementation chỉ đi theo approved contract.
- Evaluation cần evidence thật, không chỉ đọc code.
- `.harness/project/*` mô tả target repo và có thể được target repo chỉnh sửa.

## Ownership

Sau khi install, target repository sở hữu `.harness/`. Repo seed không kiểm soát target repo và không tự đồng bộ ngược thay đổi.
