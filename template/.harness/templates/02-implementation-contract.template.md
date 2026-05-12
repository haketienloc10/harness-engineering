# Implementation Contract

## Runtime Metadata

```yaml
role: Planner
runtime_mode: production_multi_session
independence: independent
contract_author_session_id: <required>
```

## Classification Guard

- Task classification: Normal Run | Epic Child Run
- Normal run is bounded: yes | no
- Oversized/Epic signals present: no | yes
- If oversized/Epic signals are present, this contract must be rejected unless scoped down:

## Contract Status

Draft

## Goal

<Generator hiểu task này cần làm gì>

## Planned Change

- ...

## Non-Goals

- ...

## Files / Areas Expected to Change

| Area/File | Expected Change | Reason | Conflict Risk |
|---|---|---|---|
|  |  |  | Low/Medium/High |

## Conflict Check

Active runs checked:

- [ ] Yes
- [ ] No

Potential conflicts:

| Run ID | File/Area | Conflict | Decision |
|---|---|---|---|
|  |  |  | Continue / Sequence / Worktree / Block |

## Behaviour Contract

Sau khi implement, hệ thống phải có các hành vi sau:

- [ ] Behaviour 1:
- [ ] Behaviour 2:
- [ ] Behaviour 3:

## Verification Plan

Evaluator có thể kiểm chứng bằng:

```bash
# Build
...

# Unit tests
...

# Integration tests
...

# Smoke test
...
```

## Manual / Runtime Checks

Nếu cần chạy app thật:

```bash
# Start app
...

# Check endpoint/page
...
```

## Rollback / Safety Notes

- ...

## Questions / Assumptions

- Assumption 1:
- Assumption 2:
