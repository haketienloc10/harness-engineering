---
artifact: 05-evaluator-report
run_id: <RUN-ID>
role: evaluator
executor_type:
executor_id:
template_source: .harness/subagents/evaluator.md
status: draft
generator_executor_id:
evaluator_executor_id:
same_executor_as_generator: false
---

# Evaluator Report

## Independence Check

- Evaluator executor is separate from Generator executor: yes | no
- Evaluator was spawned from `.harness/subagents/evaluator.md`: yes | no
- Decision if not independent or not template-based: fail

## Coordinator-Readable Decision Summary

- Failed acceptance criteria:
- Required responsible role: planner | generator | evaluator
- Required recheck:

## What Was Evaluated

- Planner brief:
- Implementation contract:
- Code diff:
- Runtime behaviour:
- Tests:

## Commands Executed

```bash
```

## Evidence

Include exact commands, outputs, logs, screenshots descriptions, browser/API evidence, or runtime observations.

```text
```

## Decision

- Reason:

## Notes for Generator

<Chi ghi yeu cau fix, khong sua code>

- Status: <required: pass or fail or blocked_insufficient_evidence>
