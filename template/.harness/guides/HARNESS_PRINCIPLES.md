# Harness Principles

Harness giúp AI-assisted development có thể audit và verify được.

## Nguyên tắc

- Mỗi task không tầm thường phải có run artifact.
- Planner, Generator, và Evaluator là vai trò tách biệt dù cùng một agent thực hiện.
- Implementation chỉ đi theo approved contract.
- Evaluation cần evidence thật, không chỉ đọc code.
- `.harness/project/*` mô tả target repo và có thể được target repo chỉnh sửa.

## Ownership

Sau khi install, target repository sở hữu `.harness/`. Repo seed không kiểm soát target repo và không tự đồng bộ ngược thay đổi.
