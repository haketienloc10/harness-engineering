# Subagent Execution

Harness is agent-runtime agnostic.

Subagents, task tools, external agent sessions, or isolated role workers are treated as independent role executors.

Independent role executors are mandatory when the current runtime supports them.

They are not the workflow state machine. `run.yaml` remains the authoritative state source.

## Runtime Capability Rule

Before executing production role work, determine whether the current runtime supports any independent execution capability:

- subagent
- task tool
- delegated worker
- external agent session
- isolated role executor
- role-specific process

If supported, the Orchestrator MUST dispatch the next lifecycle role to an independent role executor.

If unsupported, the Orchestrator MUST create `HANDOFF.md`, set state to `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, and stop.

## Required Executor Roles

- `planner`
- `contract-reviewer`
- `generator`
- `evaluator`

## Hard Rules

1. When independent role execution is available, the Orchestrator MUST dispatch the required role-specific executor.
2. The Orchestrator MUST NOT perform required role work directly.
3. The Orchestrator controls lifecycle routing and state transitions only.
4. `run.yaml` and required artifacts control the workflow state.
5. Contract Reviewer must not implement code.
6. Generator must not evaluate its own output.
7. Evaluator must verify with real evidence and must not patch implementation to make tests pass.
8. Evaluator must be independent from Generator.
9. Contract Reviewer must be independent from Planner.
10. If independent role execution is unavailable, create `HANDOFF.md`, set state to `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, and stop.
11. Do not create `HANDOFF.md` when the next role can be executed by an independent role executor.

## Orchestrator Duties

- Read `run.yaml`.
- Determine the next required state, role, and artifact.
- Dispatch the correct role-specific executor when available.
- Provide the executor only the visible inputs required for that role.
- Refuse invalid transitions.
- Update `run.yaml` only after the required artifact exists.
- Never invent approval, implementation, or verification decisions.

## Role Dispatch Table

| Lifecycle State | Required Executor | Required Output |
|---|---|---|
| `PLANNING` | `planner` | `01-planner-brief.md` |
| `CONTRACTING` | `planner` | `02-implementation-contract.md` |
| `CONTRACT_REVIEW` | `contract-reviewer` | `03-evaluator-contract-review.md` |
| `GENERATING` | `generator` | code changes + `04-generator-worklog.md` |
| `EVALUATING` | `evaluator` | `05-evaluator-report.md` |
| `FAILED_VERIFICATION` | `generator`, then `evaluator` | `06-fix-report.md`, then updated `05-evaluator-report.md` |

## Role Boundaries

| Role | Owns | Must Not Do |
|---|---|---|
| `planner` | `01-planner-brief.md`, `02-implementation-contract.md` | Implement code, approve own contract |
| `contract-reviewer` | `03-evaluator-contract-review.md` | Implement code, rewrite contract silently |
| `generator` | Implementation changes, `04-generator-worklog.md`, fix notes for `06-fix-report.md` | Approve contract, evaluate own work |
| `evaluator` | `05-evaluator-report.md`, verification evidence | Patch implementation, rely on hidden reasoning |

## Executor Type

Artifacts and `run.yaml` should record the executor type:

- `subagent`
- `task_tool`
- `external_agent_session`
- `isolated_process`
- `manual_handoff`
- `fallback_single_session`

`fallback_single_session` is not production-grade unless explicitly allowed by run policy.

## Handoff Rule

`HANDOFF.md` is only for runtimes where the next independent executor cannot be started.

If an independent role executor can be started, use the next executor instead of creating `HANDOFF.md`.
