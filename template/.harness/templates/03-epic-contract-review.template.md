# Epic Contract Review

## Runtime Metadata

```yaml
role: ContractReviewer
runtime_mode: codex_project_subagents_required
executor_type: subagent
executor_id: <required>
agent_runtime: <required>
agent_session_id: <required>
role_agent_name: harness_contract_reviewer
role_agent_file: .codex/agents/harness-contract-reviewer.toml
independence: independent
epic_planner_executor_id: <required>
```

## Epic ID

<EPIC-ID>

## Independence Check

- Reviewer executor is separate from Epic planner/coordinator executor: yes | no
- Reviewer was spawned from `.codex/agents/harness-contract-reviewer.toml`: yes | no
- Decision if not independent or not Codex-agent-based: REJECTED

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
