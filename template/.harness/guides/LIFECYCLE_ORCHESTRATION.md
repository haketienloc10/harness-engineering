# Lifecycle Orchestration

Harness workflow is a strict template-based subagent lifecycle, not a generic summary protocol:

```txt
Orchestrator -> Planner -> Contract Reviewer -> Generator -> Evaluator
```

`run.yaml` and `run-manifest.md` are the authoritative state and audit sources. Markdown artifacts are evidence for state transitions.

## Template-Based Subagent Dispatch Rule

Harness uses spawned subagents for role transitions.

The coordinator must instantiate each required lifecycle role from the fixed template in `.harness/subagents/`.

The coordinator may pass task-specific inputs, but must not write free-form prompts for core roles or modify template responsibilities, output schema, evidence requirements, or pass/fail criteria.

Required dispatch:

- `PLANNING` -> spawn `.harness/subagents/planner.md`
- `CONTRACTING` -> spawn `.harness/subagents/planner.md`
- `CONTRACT_REVIEW` -> spawn `.harness/subagents/contract-reviewer.md`
- `GENERATING` -> spawn `.harness/subagents/generator.md`
- `EVALUATING` -> spawn `.harness/subagents/evaluator.md`
- `FAILED_VERIFICATION` -> spawn `.harness/subagents/generator.md`, then `.harness/subagents/evaluator.md`

If the runtime cannot spawn subagents, the run must enter `BLOCKED_FOR_EXECUTOR_UNAVAILABLE` before Planner execution. There is no degraded single-session fallback.

Required blocked message:

```text
Subagent runtime unavailable.
Harness lifecycle requires template-based subagent orchestration.
This run is blocked.
No lifecycle role may be executed in this session.
```

## Hard Rules

1. Generator cannot start unless Contract Reviewer approved the contract.
2. Evaluator must be a separate spawned subagent from Generator.
3. Final Summary cannot claim completion without evaluator evidence.
4. The Orchestrator may coordinate state but must not replace required role decisions with its own judgment.
5. If a required spawned subagent cannot run, set state to `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, update `run-manifest.md`, and stop.
6. The coordinator must not create role artifacts on behalf of Planner, Contract Reviewer, Generator, or Evaluator.
7. Role transitions must not create `HANDOFF.md`.

## State Table

| State | Required Input Artifacts | Required Role Executor | Allowed Actions | Forbidden Actions | Required Output Artifact | Allowed Next States |
|---|---|---|---|---|---|---|
| `CREATED` | `run.yaml`, `run-manifest.md`, `00-input.md` | Coordinator | Confirm task record, check subagent runtime, spawn Planner | Implement code, approve contract, evaluate, emulate Planner | Updated `run.yaml` and `run-manifest.md` | `PLANNING`, `CANCELLED`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE` |
| `PLANNING` | `00-input.md`, project/codebase context as needed | `planner` | Analyze scope, classify run, write plan | Implement code, approve own contract, evaluate | `01-planner-brief.md` | `CONTRACTING`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `CONTRACTING` | `01-planner-brief.md` | `planner` | Write measurable implementation contract | Implement code, approve own contract, evaluate | `02-implementation-contract.md` | `CONTRACT_REVIEW`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `CONTRACT_REVIEW` | `02-implementation-contract.md` | `contract-reviewer` | Approve or reject contract, document gaps | Implement code, rewrite contract silently | `03-contract-review.md` | `APPROVED_FOR_IMPLEMENTATION`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `APPROVED_FOR_IMPLEMENTATION` | Approved `03-contract-review.md`, `approved_for_implementation: true`, `generator_allowed: true` | Coordinator | Spawn Generator | Start evaluation, claim implementation done, generate code directly | Updated `run.yaml` and `run-manifest.md` | `GENERATING`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `GENERATING` | Approved contract and review artifacts | `generator` | Implement only the approved contract, record commands and diff summary | Change contract scope, self-evaluate, mark complete | `04-implementation-report.md` | `EVALUATING`, `FAILED_VERIFICATION`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `EVALUATING` | `04-implementation-report.md`, code diff, verification commands | `evaluator` | Verify with real evidence, pass/fail/block | Patch implementation to make tests pass, rely on hidden memory, be same executor as Generator | `05-evaluator-report.md` | `COMPLETED`, `FAILED_VERIFICATION`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `COMPLETED` | Passing `05-evaluator-report.md` with command/evidence sections | Orchestrator or Evaluator | Write final summary based on evaluator evidence | Claim completion without evaluator report | `07-final-summary.md` | None |
| `REJECTED_FOR_REPLAN` | Rejection in contract review or evaluation | `planner` | Revise plan/contract within run scope | Implement before reapproval | Updated `01-planner-brief.md` and/or `02-implementation-contract.md` | `CONTRACT_REVIEW`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `BLOCKED_FOR_EXECUTOR_UNAVAILABLE` | Current `run.yaml`, `run-manifest.md`, `blocked_reason` | Coordinator | Report missing subagent runtime | Use handoff files, continue required role work in same session, write role artifacts | Updated `run.yaml` and `run-manifest.md` with block reason | None until a runtime with subagent spawning is available |
| `FAILED_VERIFICATION` | Failing `05-evaluator-report.md` | `generator` for fixes, then `evaluator` for recheck | Fix only verified failures, document fix | Evaluator patches implementation, Generator approves own fix | `06-fix-report.md` then updated `05-evaluator-report.md` | `EVALUATING`, `COMPLETED`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `CANCELLED` | Cancellation reason | Orchestrator | Record cancellation | Continue implementation or evaluation | Updated `run.yaml` and optional note in `07-final-summary.md` | None |

## State Fields

Update these `run.yaml` fields at each transition:

```yaml
state:
current_role:
updated_at:
approved_for_implementation:
generator_allowed:
last_artifact:
next_required_artifact:
next_required_role:
next_required_executor:
role_templates:
role_executors:
run_manifest:
blocked_reason:
```

The Orchestrator may update state fields after a role artifact exists. It must not invent approval, implementation, or verification decisions.
