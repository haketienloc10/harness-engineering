# Epic Contract Review

## Runtime Metadata

```yaml
role: ContractReviewer
runtime_mode: production_multi_session
independence: independent
reviewer_session_id: <required>
epic_planner_session_id: <required>
```

## Epic ID

<EPIC-ID>

## Independence Check

- Reviewer is separate from Epic planner/coordinator: yes | no
- If no, is fallback explicitly allowed for this task: yes | no
- Decision if not independent and fallback not allowed: REJECTED

## Decision

- Status: APPROVED | REJECTED
- Reason:

## Epic Quality Checklist

- Epic classification is correct: pass | fail
- Task must not be a normal run: pass | fail
- At least two independently verifiable child runs are identified: pass | fail
- Child runs have bounded objectives: pass | fail
- Epic acceptance criteria can be mapped to child-run evidence: pass | fail
- Child-run role separation is required: pass | fail
- Epic coordinator does not replace child-run review/evaluation: pass | fail

## Required Revisions

Required if rejected.

- ...
