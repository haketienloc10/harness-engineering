# Lifecycle Orchestration

Harness workflow is a role lifecycle, not a generic summary protocol:

```txt
Planner -> Contract Reviewer -> Generator -> Evaluator
```

`run.yaml` is the authoritative state source. Markdown artifacts are evidence for state transitions.

## Hard Rules

1. Generator cannot start unless Contract Reviewer approved the contract.
2. Evaluator cannot be the same executor as Generator.
3. Final Summary cannot claim completion without evaluator evidence.
4. The orchestrator may coordinate state but must not replace required role decisions with its own judgment.
5. If a required executor cannot run, create `HANDOFF.md`, set state to `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, and stop.

## State Table

| State | Required Input Artifacts | Required Role Executor | Allowed Actions | Forbidden Actions | Required Output Artifact | Allowed Next States |
|---|---|---|---|---|---|---|
| `CREATED` | `run.yaml`, `00-input.md` | Orchestrator | Confirm task record, route to Planner | Implement code, approve contract, evaluate | Updated `run.yaml` with `state: PLANNING` | `PLANNING`, `CANCELLED`, `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF` |
| `PLANNING` | `00-input.md`, project/codebase context as needed | `planner` | Analyze scope, classify run, write plan | Implement code, approve own contract, evaluate | `01-planner-brief.md` | `CONTRACTING`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, `CANCELLED` |
| `CONTRACTING` | `01-planner-brief.md` | `planner` | Write measurable implementation contract | Implement code, approve own contract, evaluate | `02-implementation-contract.md` | `CONTRACT_REVIEW`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, `CANCELLED` |
| `CONTRACT_REVIEW` | `02-implementation-contract.md` | `contract_reviewer` | Approve or reject contract, document gaps | Implement code, rewrite contract silently | `03-evaluator-contract-review.md` | `APPROVED_FOR_IMPLEMENTATION`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, `CANCELLED` |
| `APPROVED_FOR_IMPLEMENTATION` | Approved `03-evaluator-contract-review.md`, `approved_for_implementation: true`, `generator_allowed: true` | Orchestrator | Route to Generator | Start evaluation, claim implementation done | Updated `run.yaml` with `state: GENERATING` | `GENERATING`, `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, `CANCELLED` |
| `GENERATING` | Approved contract and review artifacts | `generator` | Implement only the approved contract, record commands and diff summary | Change contract scope, self-evaluate, mark complete | `04-generator-worklog.md` | `EVALUATING`, `FAILED_VERIFICATION`, `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, `CANCELLED` |
| `EVALUATING` | `04-generator-worklog.md`, code diff, verification commands | `evaluator` | Verify with real evidence, pass/fail/block | Patch implementation to make tests pass, rely on hidden memory, be same executor as Generator | `05-evaluator-report.md` | `COMPLETED`, `FAILED_VERIFICATION`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, `CANCELLED` |
| `COMPLETED` | Passing `05-evaluator-report.md` with command/evidence sections | Orchestrator or Evaluator | Write final summary based on evaluator evidence | Claim completion without evaluator report | `07-final-summary.md` | None |
| `REJECTED_FOR_REPLAN` | Rejection in contract review or evaluation | `planner` | Revise plan/contract within run scope | Implement before reapproval | Updated `01-planner-brief.md` and/or `02-implementation-contract.md` | `CONTRACT_REVIEW`, `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, `CANCELLED` |
| `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF` | Current `run.yaml` and latest artifact | Orchestrator | Create `HANDOFF.md` for next independent executor | Continue required role work in same executor | `HANDOFF.md` | External session resumes to the required next state |
| `FAILED_VERIFICATION` | Failing `05-evaluator-report.md` | `generator` for fixes, then `evaluator` for recheck | Fix only verified failures, document fix | Evaluator patches implementation, Generator approves own fix | `06-fix-report.md` then updated `05-evaluator-report.md` | `EVALUATING`, `COMPLETED`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF`, `CANCELLED` |
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
role_executors:
```

The orchestrator may update state fields after a role artifact exists. It must not invent approval, implementation, or verification decisions.
