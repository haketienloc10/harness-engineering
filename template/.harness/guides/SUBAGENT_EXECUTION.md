# Codex Subagent Execution

Subagents are lifecycle role executors. They are not the workflow, not the state machine, and not the source of truth.

## Required Executor Roles

- `planner`
- `contract-reviewer`
- `generator`
- `evaluator`

## Rules

1. When Codex subagents are available, use role-specific subagents as lifecycle executors.
2. The orchestrator controls state transitions only.
3. `run.yaml` and required artifacts control the workflow state.
4. Contract Reviewer must not implement code.
5. Generator must not evaluate its own output.
6. Evaluator must verify with real evidence and must not patch implementation to make tests pass.
7. Evaluator must be independent from Generator.
8. If subagents are unavailable, create `HANDOFF.md`, set state to `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, and stop.

## Orchestrator Duties

- Read `run.yaml`.
- Determine the next required role and artifact.
- Start the correct role executor when available.
- Update `run.yaml` only after the required artifact exists.
- Refuse invalid transitions.
- Create `HANDOFF.md` when independent role execution cannot continue.

## Role Boundaries

| Role | Owns | Must Not Do |
|---|---|---|
| `planner` | `01-planner-brief.md`, `02-implementation-contract.md` | Implement code, approve own contract |
| `contract-reviewer` | `03-evaluator-contract-review.md` | Implement code, rewrite contract silently |
| `generator` | Implementation changes, `04-generator-worklog.md`, fix notes for `06-fix-report.md` | Approve contract, evaluate own work |
| `evaluator` | `05-evaluator-report.md`, verification evidence | Patch implementation, rely on hidden reasoning |

The lifecycle state machine remains authoritative even when several subagents are used in parallel for exploration. Parallel work must feed visible artifacts before any state transition.
