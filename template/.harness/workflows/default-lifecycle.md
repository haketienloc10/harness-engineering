# Default Lifecycle

This file defines state transitions only. Role policy lives in `.codex/agents/*.toml`; orchestration policy lives in `.harness/guides/SUBAGENT_EXECUTION.md`; workflow skills live in `.codex/skills/harness-*/SKILL.md`.

## State Transitions

| State | Required Role | Inputs | Required Outputs | Next State |
|---|---|---|---|---|
| `CREATED` | coordinator | user request | `00-input.md`, baseline metadata | `PLANNING` |
| `PLANNING` | `harness_planner` | `00-input.md`, relevant project/codebase context | `01-planner-brief.md`, `02-implementation-contract.md` | `CONTRACT_REVIEW` or `REJECTED_FOR_REPLAN` |
| `CONTRACT_REVIEW` | `harness_contract_reviewer` | `01-planner-brief.md`, `02-implementation-contract.md` | `03-contract-review.md` | `APPROVED_FOR_IMPLEMENTATION` or `REJECTED_FOR_REPLAN` |
| `REJECTED_FOR_REPLAN` | `harness_planner` | reviewer feedback and prior planning artifacts | updated `01-planner-brief.md`, `02-implementation-contract.md` | `CONTRACT_REVIEW` |
| `APPROVED_FOR_IMPLEMENTATION` | coordinator | approved `03-contract-review.md` | generator dispatch metadata | `GENERATING` |
| `GENERATING` | `harness_generator` | approved contract and review | code changes, `04-implementation-report.md` | `EVALUATING` |
| `EVALUATING` | `harness_evaluator` | full artifact chain, diff, command/runtime evidence | `05-evaluator-report.md` | `COMPLETED` or `FAILED_VERIFICATION` |
| `FAILED_VERIFICATION` | `harness_generator`, then `harness_evaluator` | evaluator decision summary and rework packet | updated `04-implementation-report.md`, updated `05-evaluator-report.md` | `COMPLETED` or `FAILED_VERIFICATION` |
| `BLOCKED_FOR_EXECUTOR_UNAVAILABLE` | none | runtime capability record | blocked reason | none |
| `CANCELLED` | coordinator | cancellation reason | updated `run.yaml` | none |

## Required Final Summary

After `05-evaluator-report.md` passes, coordinator may write `06-final-summary.md` only as an aggregate of:

```txt
05-evaluator-report.md
04-implementation-report.md
run.yaml
```

## Runtime Capability

New runs start with:

```yaml
runtime:
  subagent_runtime_available: unknown
```

Use:

```bash
bash .harness/scripts/set-runtime-capability.sh .harness/runs/<RUN_ID> true
```

This records a manual assertion. It is not proof that the runtime can spawn Codex agents.
