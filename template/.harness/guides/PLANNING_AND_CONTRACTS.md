# Planning and Contract Policy

## Epic Check

Trước khi tạo một run, classify task bằng `.harness/guides/RUN_CLASSIFICATION.md`. Nếu task có nhiều phase, milestone, user flow, module, scope chưa chắc, hoặc không thể verify gọn trong một run, dùng `.harness/guides/LONG_TASK_POLICY.md` và tạo Epic. Một task có wording như `phase 1-4`, `part 1-4`, `core loop`, `complete playable`, `full feature`, `end-to-end`, `MVP`, `large task`, hoặc `long task` không được trở thành một normal run.

Nếu task đủ lớn để thành Epic nhưng mới biết một child run, Planner phải giảm scope xuống một normal run thật sự bounded, hoặc hỏi/derive thêm decomposition trước implementation.

## Planner Brief

Planner subagent là role duy nhất authored `01-planner-brief.md`. Planner subagent phải được spawned từ `.codex/agents/harness-planner.toml` và không được implement application code, approve contract, hoặc approve evaluation của chính run đó.

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

`02-implementation-contract.md` chỉ được authored bởi Planner subagent. Planner subagent không được approve contract của chính mình.

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

`03-contract-review.md` chỉ được authored bởi Contract Reviewer subagent spawned từ `.codex/agents/harness-contract-reviewer.toml`. Contract Reviewer phải output `approved` hoặc `rejected_requires_revision` rõ ràng. Không sửa application code trước khi contract được approve.

The Contract Reviewer must not be the same runtime session that authored the contract.

The Contract Reviewer must reject the contract if the reviewer is the same runtime session as the contract author.

The Contract Reviewer must reject any normal run contract that should be an Epic.

Contract Reviewer phải reject nếu:

- acceptance criteria bị thiếu hoặc không measurable;
- verification plan bị thiếu hoặc không executable;
- affected files/areas quá vague;
- behaviour contract ambiguous;
- assumptions không được nêu rõ;
- scope quá lớn cho một run;
- too many phases in one normal run;
- contract conflict với project rules hoặc user request;
- không có independent verification path;
- role separation violation;
- contract cần hidden assumptions từ Planner để hiểu hoặc verify.

Contract Reviewer không được silently rewrite contract. Nếu cần sửa, ghi `Status: rejected_requires_revision` và liệt kê `Required Revisions` để Planner subagent revise trước implementation.

## Backward Compatibility

Existing old runs có thể không có runtime metadata. New runs phải include metadata và independence checks theo templates hiện tại. Không rewrite old runs trừ khi user yêu cầu rõ.
