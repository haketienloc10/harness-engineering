---
artifact: 03-contract-review
run_id: <RUN-ID>
role: contract_reviewer
executor_type:
executor_id:
template_source: .harness/subagents/contract-reviewer.md
status: draft
---

# Contract Reviewer Report

## Independence Check

- Reviewer executor is separate from planner executor: yes | no
- Reviewer was spawned from `.harness/subagents/contract-reviewer.md`: yes | no
- Decision if not independent or not template-based: rejected_requires_revision

## Decision

- Status: approved | rejected_requires_revision
- Reason:

## Contract Quality Checklist

- Task classification is correct: pass | fail
- Acceptance criteria are measurable: pass | fail
- Verification plan is executable: pass | fail
- Scope is bounded: pass | fail
- Behaviour contract is clear: pass | fail
- Independent verification path exists: pass | fail

## Issues Found

| Severity | Issue | Required Revision |
|---|---|---|
| High/Medium/Low |  |  |

## Dispatch Decision

- Next role allowed to proceed: generator | planner | none
- Required next executor:
- Required next state:
- Reason:
