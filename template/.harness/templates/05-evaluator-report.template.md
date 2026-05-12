# Evaluator Report

## Runtime Metadata

- Role: Evaluator Agent
- Evaluator session: <session id / agent id / manual label>
- Generator session reviewed: <session id / agent id / manual label>
- Runtime mode: production_multi_session | fallback_single_session
- Independence: independent | degraded

## Independence Check

- Evaluator is separate from Generator: yes | no
- If no, mark lower trust and explain fallback.

## Evaluation Decision

- [ ] Pass
- [ ] Fail
- [ ] Pass with Notes
- [ ] Blocked

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

- Status: PASS | FAIL
- Reason:

## Notes for Generator

<Chỉ ghi yêu cầu fix, không sửa code>
