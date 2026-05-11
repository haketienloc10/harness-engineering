# Planning and Contract Policy

## Planner Brief

`01-planner-brief.md` must clarify:

- mục tiêu
- phạm vi
- ngoài phạm vi
- tiêu chí nghiệm thu
- khu vực có khả năng bị ảnh hưởng
- rủi ro và điểm chưa rõ

The planner should not over-spec implementation.

Avoid forcing early:

- class names
- method names
- component names
- SQL structure
- unnecessary dependencies
- premature architectural decisions

## Implementation Contract

`02-implementation-contract.md` must include:

- mục tiêu
- thay đổi dự kiến
- ngoài phạm vi
- file hoặc khu vực dự kiến thay đổi
- rủi ro conflict
- behaviour contract
- kế hoạch kiểm chứng
- giả định

If the contract is vague, not measurable, or not testable, revise it before coding.

## Contract Review

`03-evaluator-contract-review.md` must explicitly approve or reject the contract.

Application code must not be modified before contract approval.
