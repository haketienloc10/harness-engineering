# Evaluator Report

## Executor Metadata

```yaml
role: Evaluator
runtime_mode: template_subagents_required
executor_type: subagent
executor_id: <required>
agent_runtime: <required>
agent_session_id: <required>
template_source: .harness/subagents/evaluator.md
started_at: <required>
completed_at: <required>
independence: independent
generator_executor_id: <required>
evaluator_executor_id: <required>
same_executor_as_generator: false
```

## Independence Check

- Evaluator executor is separate from Generator executor: yes | no
- Evaluator was spawned from `.harness/subagents/evaluator.md`: yes | no
- Decision if not independent or not template-based: fail

## Evaluation Decision

- [ ] pass
- [ ] fail
- [ ] blocked_insufficient_evidence

## What Was Evaluated

- Planner brief:
- Implementation contract:
- Code diff:
- Runtime behaviour:
- Tests:
- Conflict status:

## Commands Executed

```bash
<command>
```

### Result

```text
<paste output summary>
```

## Runtime / App Checks

| Check | Method | Result | Evidence |
|---|---|---|---|
|  | CLI/API/Browser | Pass/Fail |  |

## Behaviour-Level Evidence

Evaluator phải điền một dòng cho từng required behaviour trong implementation contract. Với UI task, không được `Pass` nếu chỉ có build success hoặc curl smoke mà thiếu evidence cho các behaviour bắt buộc.

| Behaviour | Kỳ vọng | Phương pháp kiểm chứng | Evidence | Kết quả |
|---|---|---|---|---|
|  |  | Browser/E2E/Manual/API/Other |  | Pass/Fail |

## Behaviour Verification Summary

| Behaviour | Expected | Actual | Result |
|---|---|---|---|
|  |  |  | Pass/Fail |

## Conflict Verification

| Check | Result | Evidence |
|---|---|---|
| Modified files match contract | Pass/Fail |  |
| No overlap with active run | Pass/Fail |  |
| Branch/worktree isolation respected | Pass/Fail/NA |  |

## Bugs / Issues

| Severity | Issue | Evidence | Suggested Fix |
|---|---|---|---|
| High/Medium/Low |  |  |  |

## Missing Tests

- ...

## Evidence

Include exact commands, outputs, logs, screenshots descriptions, browser/API evidence, or runtime observations.

```text
...
```

## Decision

- Status: pass | fail | blocked_insufficient_evidence
- Reason:

## Notes for Generator

<Chỉ ghi yêu cầu fix, không sửa code>
