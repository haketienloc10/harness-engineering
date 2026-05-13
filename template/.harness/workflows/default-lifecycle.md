# Default Lifecycle

## Required Execution Model

Harness core lifecycle roles must run as separate spawned subagents from fixed templates under `.harness/subagents/`.

There is no degraded single-session fallback.

The Coordinator is orchestration-only. It must not implement, debug, repair, review, verify, approve, edit source/tests/config, or write role artifacts directly.

`dispatch-role.sh` creates a dispatch artifact only. It does not spawn or execute a subagent. A runtime executor must consume `.harness/runs/<RUN_ID>/dispatch/<role>.dispatch.md` and spawn the role-specific subagent from `.harness/subagents/<role>.md`.

## Steps

1. Coordinator starts run.
2. Coordinator checks subagent runtime availability.
3. If unavailable, set `runtime.subagent_runtime_available: false`, update `run-manifest.md`, and block run immediately.
4. If available, set `runtime.subagent_runtime_available: true`, update `run-manifest.md`, call `dispatch-role.sh <run> planner`, and require the runtime executor to spawn Planner from `.harness/subagents/planner.md`.
5. Planner writes `01-planner-brief.md`.
6. Coordinator prepares implementation contract routing; Planner writes `02-implementation-contract.md` when the workflow enters `CONTRACTING`.
7. Call `dispatch-role.sh <run> contract_reviewer` and spawn Contract Reviewer from `.harness/subagents/contract-reviewer.md`.
8. Contract Reviewer writes `03-contract-review.md`.
9. If contract rejected, return to Planner/contract revision.
10. If approved, call `dispatch-role.sh <run> generator` and spawn Generator from `.harness/subagents/generator.md`.
11. Generator writes `04-implementation-report.md`.
12. Call `dispatch-role.sh <run> evaluator` and spawn Evaluator from `.harness/subagents/evaluator.md`.
13. Evaluator writes `05-evaluator-report.md`.
14. If Evaluator returns a non-passing result, Coordinator reads only the evaluator decision summary, creates a bounded Generator rework packet from `.harness/templates/generator-rework-packet.template.md`, spawns Generator, then spawns Evaluator again.
15. Run completes only if Evaluator result is `pass`.

## Created Manifest State

New runs start before executor availability has been checked. `run.yaml` starts with:

```yaml
runtime:
  subagent_runtime_available: unknown
```

`run-manifest.md` starts with:

```md
- mode: template_subagents_required
- dispatch_mode: template_based
- fallback_allowed: false
- subagent_runtime_available: unknown
- run_status: created_pending_executor_check
```

After a successful runtime check, update it before dispatching Planner:

```yaml
runtime:
  subagent_runtime_available: true
```

```md
- subagent_runtime_available: true
- run_status: ready_for_planner_dispatch
```

Use the helper when available:

```bash
bash .harness/scripts/mark-subagent-runtime.sh .harness/runs/<RUN_ID> true
```

## Block Rule

If subagent runtime is unavailable, create or update `run-manifest.md`:

```md
# Run Manifest

## Execution Mode

- mode: template_subagents_required
- dispatch_mode: template_based
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

## Rework Block Rule

If implementation or rework is required and Generator cannot be spawned, the coordinator must stop with:

```text
BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
```

The coordinator must not fall back to direct implementation.
