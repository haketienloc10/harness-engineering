# Planning and Contract Policy

## Epic Check

Trước khi tạo một run, quyết định task có thuộc Epic container không. Nếu task có nhiều milestone, nhiều user flow, nhiều module, scope chưa chắc, hoặc không thể verify gọn trong một run, dùng `.harness/guides/LONG_TASK_POLICY.md` và chia thành nhiều child runs nhỏ. Chỉ tạo Epic khi planner xác định được ít nhất hai child runs có thể verify độc lập; nếu chỉ biết một run, tạo normal run và ghi follow-up proposal/backlog.

## Planner Brief

Planner Agent là role duy nhất authored `01-planner-brief.md` trong production workflow. Planner Agent không được implement application code, approve contract, hoặc approve evaluation của chính run đó.

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

`02-implementation-contract.md` chỉ được authored bởi Planner Agent. Planner Agent không được approve contract của chính mình.

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

`03-evaluator-contract-review.md` chỉ được authored bởi Contract Reviewer Agent. Contract Reviewer Agent phải approve hoặc reject rõ ràng. Không sửa application code trước khi contract được approve.

The Contract Reviewer must not be the same runtime session that authored the contract for production-grade workflow.

Contract Reviewer phải reject nếu:

- acceptance criteria bị thiếu hoặc không measurable;
- verification plan bị thiếu hoặc không executable;
- affected files/areas quá vague;
- behaviour contract ambiguous;
- assumptions không được nêu rõ;
- scope quá lớn cho một run;
- contract conflict với project rules hoặc user request;
- không có independent verification path;
- contract cần hidden assumptions từ Planner để hiểu hoặc verify.

Contract Reviewer không được silently rewrite contract. Nếu cần sửa, ghi `Status: REJECTED` và liệt kê `Required Revisions` để Planner Agent revise trước implementation.

## Backward Compatibility

Existing old runs có thể không có runtime metadata. New runs phải include metadata và independence checks theo templates hiện tại. Không rewrite old runs trừ khi user yêu cầu rõ.
