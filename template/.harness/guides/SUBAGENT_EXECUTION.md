# Subagent Execution

Harness is agent-runtime agnostic.

Role transitions are performed by spawning or dispatching independent role executors.

Harness does not use `HANDOFF.md` for normal lifecycle transitions.

## Runtime Capability Rule

Before executing production role work, determine whether the current runtime supports an independent role executor capability, such as:

- subagent
- task tool
- delegated worker
- external agent session
- isolated role executor
- role-specific process

If supported, the Orchestrator MUST dispatch the next lifecycle role to the corresponding independent executor.

If unsupported, the Orchestrator MUST block the run unless `fallback_single_session_allowed: true` is explicitly set in `run.yaml`.

## Required Executors

- `planner`
- `contract-reviewer`
- `generator`
- `evaluator`

## Hard Rules

1. The Orchestrator controls lifecycle routing and state transitions only.
2. The Orchestrator MUST NOT perform required role work directly.
3. The Orchestrator MUST NOT create `HANDOFF.md` for normal role transitions.
4. The Orchestrator MUST dispatch the next role-specific executor.
5. `run.yaml` is the authoritative lifecycle state source.
6. Contract Reviewer must not implement code.
7. Generator must not evaluate its own output.
8. Evaluator must not patch implementation to make tests pass.
9. Evaluator must be independent from Generator.
10. Contract Reviewer must be independent from Planner.
11. Single-agent role simulation is not production-grade.
12. If independent execution is unavailable and fallback is not explicitly allowed, set the run state to `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`.

## Orchestrator Duties

- Read `run.yaml`.
- Determine the next required lifecycle state.
- Determine the next required role.
- Spawn or dispatch the correct role executor.
- Provide only the required visible inputs for that role.
- Refuse invalid transitions.
- Update `run.yaml` only after the required artifact exists.
- Never invent role approval, implementation, or verification decisions.

## Role Dispatch Table

| Lifecycle State | Required Executor | Required Output |
|---|---|---|
| `PLANNING` | `planner` | `01-planner-brief.md` |
| `CONTRACTING` | `planner` | `02-implementation-contract.md` |
| `CONTRACT_REVIEW` | `contract-reviewer` | `03-evaluator-contract-review.md` |
| `GENERATING` | `generator` | code changes + `04-generator-worklog.md` |
| `EVALUATING` | `evaluator` | `05-evaluator-report.md` |
| `FAILED_VERIFICATION` | `generator`, then `evaluator` | `06-fix-report.md`, then updated `05-evaluator-report.md` |

## Executor Type

Record executor metadata in `run.yaml` and role artifacts.

Allowed executor types:

- `subagent`
- `task_tool`
- `external_agent_session`
- `isolated_process`
- `fallback_single_session`

`fallback_single_session` is allowed only when explicitly enabled by run policy.

## Removed Handoff Behavior

Do not create `HANDOFF.md`.

Do not ask the user to manually continue the next role if an executor can be started.

Do not use handoff files as phase boundaries.

Lifecycle role boundaries are enforced by executor dispatch and artifact ownership.
