# Planning and Contract Policy

## Epic Check

Trước khi tạo một run, quyết định task có thuộc Epic không. Nếu task có nhiều milestone, nhiều user flow, nhiều module, scope chưa chắc, hoặc không thể verify gọn trong một run, dùng `.harness/guides/LONG_TASK_POLICY.md` và chia thành nhiều run nhỏ.

## Planner Brief

`01-planner-brief.md` phải làm rõ:

- mục tiêu;
- phạm vi;
- ngoài phạm vi;
- acceptance criteria;
- khu vực có khả năng bị ảnh hưởng;
- rủi ro và điểm chưa rõ.

Planner không nên over-spec implementation.

Tránh ép sớm:

- class names;
- method names;
- component names;
- SQL structure;
- dependency không cần thiết;
- quyết định kiến trúc chưa có evidence.

## Implementation Contract

`02-implementation-contract.md` phải có:

- mục tiêu;
- thay đổi dự kiến;
- ngoài phạm vi;
- file hoặc khu vực dự kiến thay đổi;
- rủi ro conflict;
- behaviour contract;
- kế hoạch kiểm chứng;
- giả định.

Nếu contract mơ hồ, không đo được, hoặc không test được, revise trước khi coding.

## Contract Review

`03-evaluator-contract-review.md` phải approve hoặc reject rõ ràng. Không sửa application code trước khi contract được approve.
