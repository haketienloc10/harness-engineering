# Contract Reviewer Report

## Executor Metadata

```yaml
role: ContractReviewer
runtime_mode: template_subagents_required
executor_type: subagent
executor_id: <required>
agent_runtime: <required>
agent_session_id: <required>
template_source: .harness/subagents/contract-reviewer.md
started_at: <required>
completed_at: <required>
independence: independent
contract_author_executor_id: <required>
```

## Independence Check

- Reviewer executor is separate from contract author executor: yes | no
- Reviewer was spawned from `.harness/subagents/contract-reviewer.md`: yes | no
- Decision if not independent or not template-based: rejected_requires_revision

## Decision

- Status: approved | rejected_requires_revision
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

## Dispatch Decision

- Next role allowed to proceed: generator | planner | none
- Required next executor:
- Required next state:
- Reason:
