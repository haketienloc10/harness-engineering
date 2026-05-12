# Run Input

## Runtime Metadata

```yaml
role: Planner
runtime_mode: template_subagents_required
executor_type: subagent
executor_id: <required>
agent_runtime: <required>
agent_session_id: <required>
role_template: .harness/subagents/planner.md
independence: independent
```

## Task Classification

- Classification: Normal Run | Epic Child Run
- Epic required: yes | no
- If Epic required, normal run creation is invalid: yes | no
- Classification guide used: `.harness/guides/RUN_CLASSIFICATION.md`

## Run ID

<RUN-ID>

## Related Epic

None

## Source

- Manual request

## Original Request

```text
<Agent must paste or summarize the user's original request here without changing intent>
```

## Business Goal

<Mục tiêu nghiệp vụ hoặc sản phẩm cần đạt>

## Constraints

- Tech stack:
- Deadline:
- Không được thay đổi:
- Chỉ được thay đổi:

## Expected Output

- Code change:
- Test:
- Document:
- Migration:
- Other:

## Parallel / Conflict Notes

- Related runs:
- Potential conflicts:
- Branch/worktree requirement:

## Notes

<Ghi chú thêm nếu có>
