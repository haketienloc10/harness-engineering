# Contract Reviewer Report

## Runtime Metadata

```yaml
role: ContractReviewer
runtime_mode: production_multi_session
independence: independent
reviewer_session_id: <required>
contract_author_session_id: <required>
```

## Independence Check

- Reviewer is separate from contract author: yes | no
- If no, is fallback explicitly allowed for this task: yes | no
- Decision if not independent and fallback not allowed: REJECTED

## Decision

- Status: APPROVED | REJECTED
- Reason:

## Contract Quality Checklist

- Task classification is correct: pass | fail
- Normal run is not oversized: pass | fail
- Acceptance criteria are measurable: pass | fail
- Verification plan is executable: pass | fail
- Scope is bounded: pass | fail
- Behaviour contract is clear: pass | fail
- Assumptions are explicit: pass | fail
- Conflict risks are identified: pass | fail
- Project rules are respected: pass | fail
- Independent verification path exists: pass | fail

## Conflict Review

| Item | Result |
|---|---|
| Active runs checked | Yes/No |
| File overlap found | Yes/No |
| Branch/worktree needed | Yes/No |
| Decision | Continue / Sequence / Worktree / Block |

## Issues Found

| Severity | Issue | Required Revision |
|---|---|---|
| High/Medium/Low |  |  |

## Missing Verification

- ...

## Required Revisions

Only required if rejected.

- ...

## Handoff

- Next role allowed to proceed: Generator Agent | Planner Agent | None
- Reason:
