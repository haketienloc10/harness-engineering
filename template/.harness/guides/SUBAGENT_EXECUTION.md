# Subagent Execution

Harness core lifecycle execution requires template-based spawned subagents.

Role transitions are performed by spawning independent subagents from fixed templates.

Harness does not use `HANDOFF.md` for normal lifecycle transitions.

## Runtime Capability Rule

Before executing lifecycle role work, determine whether the current runtime supports real subagent spawning.

If supported, the coordinator MUST instantiate the next lifecycle role from the corresponding template under `.harness/subagents/`.

If unsupported, the coordinator MUST block the run before Planner execution.

There is no degraded single-session fallback.

Required blocked message:

```text
Subagent runtime unavailable.
Harness lifecycle requires template-based subagent orchestration.
This run is blocked.
No lifecycle role may be executed in this session.
```

## Required Templates

- `.harness/subagents/planner.md`
- `.harness/subagents/contract-reviewer.md`
- `.harness/subagents/generator.md`
- `.harness/subagents/evaluator.md`

## Hard Rules

1. The Orchestrator controls lifecycle routing and state transitions only.
2. The Orchestrator MUST NOT perform required role work directly.
3. The Orchestrator MUST NOT create `HANDOFF.md` for normal role transitions.
4. The Orchestrator MUST spawn the next role-specific subagent from its template.
5. `run.yaml` is the authoritative lifecycle state source.
6. Contract Reviewer must not implement code.
7. Generator must not evaluate its own output.
8. Evaluator must not patch implementation to make tests pass.
9. Evaluator must be independent from Generator.
10. Contract Reviewer must be independent from Planner.
11. Single-agent role execution is invalid.
12. If subagent spawning is unavailable, set the run state to `BLOCKED_FOR_EXECUTOR_UNAVAILABLE` and update `run-manifest.md`.
13. The Orchestrator MUST NOT create free-form prompts for core lifecycle roles.
14. The Orchestrator MUST NOT modify template responsibilities, forbidden actions, required artifacts, evidence requirements, or pass/fail criteria.

## Orchestrator Duties

- Read `run.yaml`.
- Read `run-manifest.md`.
- Determine the next required lifecycle state.
- Determine the next required role.
- Load the correct role template.
- Spawn the correct role subagent.
- Provide only the required visible inputs for that role.
- Refuse invalid transitions.
- Update `run.yaml` only after the required artifact exists.
- Update `run-manifest.md` for role instance status and runtime availability.
- Never invent role approval, implementation, or verification decisions.

## Role Dispatch Table

| Lifecycle State | Required Executor | Required Output |
|---|---|---|
| `PLANNING` | `planner` | `01-planner-brief.md` |
| `CONTRACTING` | `planner` | `02-implementation-contract.md` |
| `CONTRACT_REVIEW` | `contract-reviewer` | `03-contract-review.md` |
| `GENERATING` | `generator` | code changes + `04-implementation-report.md` |
| `EVALUATING` | `evaluator` | `05-evaluator-report.md` |
| `FAILED_VERIFICATION` | `generator`, then `evaluator` | `06-fix-report.md`, then updated `05-evaluator-report.md` |

## Role Template Sources

Record subagent metadata in `run.yaml`, `run-manifest.md`, and role artifacts.

Allowed executor type:

- `subagent`

No other executor type is valid for core lifecycle roles.

## Removed Handoff Behavior

Do not create `HANDOFF.md`.

Do not ask the user to manually continue the next role if an executor can be started.

Do not use handoff files as phase boundaries.

Lifecycle role boundaries are enforced by spawned subagents from fixed templates and artifact ownership.
