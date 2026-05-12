# Default Lifecycle

## Required Execution Model

Harness core lifecycle roles must run as separate spawned subagents from fixed templates under `.harness/subagents/`.

There is no degraded single-session fallback.

## Steps

1. Coordinator starts run.
2. Coordinator checks subagent runtime availability.
3. If unavailable, block run immediately.
4. If available, spawn Planner from `.harness/subagents/planner.md`.
5. Planner writes `01-planner-brief.md`.
6. Coordinator prepares implementation contract routing; Planner writes `02-implementation-contract.md` when the workflow enters `CONTRACTING`.
7. Spawn Contract Reviewer from `.harness/subagents/contract-reviewer.md`.
8. Contract Reviewer writes `03-contract-review.md`.
9. If contract rejected, return to Planner/contract revision.
10. If approved, spawn Generator from `.harness/subagents/generator.md`.
11. Generator writes `04-implementation-report.md`.
12. Spawn Evaluator from `.harness/subagents/evaluator.md`.
13. Evaluator writes `05-evaluator-report.md`.
14. Run completes only if Evaluator result is `pass`.

## Created Manifest State

New runs start before executor availability has been checked:

```md
- mode: template_subagents_required
- fallback_allowed: false
- subagent_runtime_available: unknown
- run_status: created_pending_executor_check
```

After a successful runtime check, update it before dispatching Planner:

```md
- subagent_runtime_available: true
- run_status: ready_for_planner_dispatch
```

## Block Rule

If subagent runtime is unavailable, create or update `run-manifest.md`:

```md
# Run Manifest

## Execution Mode

- mode: template_subagents_required
- fallback_allowed: false
- subagent_runtime_available: false
- run_status: blocked

## Block Reason

Subagent runtime is unavailable. Harness requires template-based subagent orchestration. This run cannot proceed.

## Required Role Instances

- planner: blocked
- contract_reviewer: blocked
- generator: blocked
- evaluator: blocked
```

The coordinator must report:

```text
Subagent runtime unavailable.
Harness lifecycle requires template-based subagent orchestration.
This run is blocked.
No lifecycle role may be executed in this session.
```
