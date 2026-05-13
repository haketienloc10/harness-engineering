---
artifact: 03-contract-review
run_id: <RUN-ID>
role: contract_reviewer
executor_type:
executor_id:
codex_agent_name: harness_contract_reviewer
codex_agent_file: .codex/agents/harness-contract-reviewer.toml
status: draft
---

# Contract Reviewer Report

## Independence Check

- Reviewer executor is separate from planner executor: yes | no
- Reviewer was spawned from `.codex/agents/harness-contract-reviewer.toml`: yes | no
- Decision if not independent or not Codex-agent-based: rejected_requires_revision

## Decision

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

- Status: <required: approved or rejected_requires_revision>
