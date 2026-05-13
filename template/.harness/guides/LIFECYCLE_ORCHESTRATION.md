# Lifecycle Orchestration

Harness workflow is a strict Codex project-scoped subagent lifecycle, not a generic summary protocol:

```txt
Orchestrator -> Planner -> Contract Reviewer -> Generator -> Evaluator
```

`run.yaml` and `run-manifest.md` are the authoritative state and audit sources. Markdown artifacts are evidence for state transitions.

## Codex Project-Scoped Subagent Dispatch Rule

Harness uses named Codex project-scoped agents for role transitions. `.harness/scripts/dispatch-role.sh` creates deterministic routing metadata before invocation.

The coordinator must call `.harness/scripts/dispatch-role.sh` for each required lifecycle role. The dispatch artifact binds the role to the Codex agent file in `.codex/agents/`.

`dispatch-role.sh` creates only `.harness/runs/<RUN_ID>/dispatch/<role>.dispatch.md`. It does not spawn or execute the role. Codex-native invocation of the named agent is the canonical execution path.

The coordinator may pass task-specific inputs, but must not write free-form prompts for core roles or modify Codex agent responsibilities, output schema, evidence requirements, or pass/fail criteria.

The coordinator is orchestration-only. It may route state, build bounded packets, invoke named Codex project-scoped agents, wait for completion, inspect role status and decision summaries, update Harness metadata, and summarize from approved artifacts. It must not implement, debug, verify, review, repair, test, or approve role work directly.

For coordinator/orchestrator sessions, the workflow MUST run `.harness/scripts/validate-coordinator-write-scope.sh` with `HARNESS_EXECUTOR_ROLE` and `HARNESS_RUN_DIR` set. The allowlist and forbidden role-owned artifacts are defined in `.harness/guides/SUBAGENT_EXECUTION.md#coordinator-write-scope-validator`. A failure blocks the workflow with `BLOCKED_COORDINATOR_WRITE_SCOPE_VIOLATION`.

Required dispatch:

- `PLANNING` -> `dispatch-role.sh <run> planner` -> invoke `harness_planner`
- `CONTRACT_REVIEW` -> `dispatch-role.sh <run> contract_reviewer` -> invoke `harness_contract_reviewer`
- `GENERATING` -> `dispatch-role.sh <run> generator` -> invoke `harness_generator`
- `EVALUATING` -> `dispatch-role.sh <run> evaluator` -> invoke `harness_evaluator`
- `FAILED_VERIFICATION` -> read evaluator decision summary only, create a bounded Generator rework packet, invoke `harness_generator`, then invoke `harness_evaluator`

New runs start with `runtime.subagent_runtime_available: unknown`. Before Planner dispatch, the coordinator must set it to `true` after confirming native subagent spawning, or set it to `false` and enter `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`. There is no degraded single-session fallback.

If a required Codex agent file is missing, invalid, or unavailable, stop with `BLOCKED_REQUIRED_CODEX_AGENT_UNAVAILABLE`.

If runtime cannot spawn a required role subagent, stop with `BLOCKED_REQUIRED_SUBAGENT_UNAVAILABLE`.

If runtime cannot spawn the required Generator for implementation or rework, stop with `BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE`.

Planner writes both `01-planner-brief.md` and `02-implementation-contract.md` in one invocation.

Required blocked message:

```text
Subagent runtime unavailable.
Harness lifecycle requires Codex project-scoped subagents from `.codex/agents/`.
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
8. The coordinator must not modify application source, tests, runtime configuration, generated production artifacts, or project implementation files.
9. The coordinator must not read implementation files for the purpose of direct repair.
10. Any source/test/config change must be performed by the Generator role.

## Evaluator Failure Routing Policy

When an Evaluator returns `FAIL`, `REJECTED`, `NEEDS_FIX`, `blocked_insufficient_evidence`, or any equivalent non-passing result, the coordinator MUST NOT fix the implementation directly.

The coordinator MUST route the failed work back to the responsible Generator role using a bounded rework packet.

The coordinator may only:

- read the evaluator decision summary;
- identify the failed run or failed acceptance criterion;
- create a Generator rework packet from `.harness/templates/generator-rework-packet.template.md`;
- spawn the Generator role;
- wait for Generator output;
- spawn the Evaluator role again.

The coordinator MUST NOT:

- edit source files;
- inspect implementation files for direct repair;
- add tests directly;
- run an implementation/debug loop;
- write a fix report as if it were the Generator.

If the runtime cannot spawn the required Generator role, the workflow MUST stop with:

```text
BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
```

## Rework Routing Policy

When a role artifact is rejected or verification fails, the coordinator MUST route the work back to the responsible role.

Routing rules:

- Contract rejected -> route back to Planner;
- Contract incomplete or ambiguous -> route back to Planner;
- Implementation violates contract -> route back to Generator;
- Source/test/config changes required -> route back to Generator;
- Verification evidence missing -> route back to Evaluator;
- Evaluator report incomplete -> route back to Evaluator;
- Acceptance criteria ambiguous -> route back to Planner.

The coordinator MUST NOT repair rejected work directly.

The coordinator MUST NOT convert itself into the missing role.

## State Table

| State | Required Input Artifacts | Required Role Executor | Allowed Actions | Forbidden Actions | Required Output Artifact | Allowed Next States |
|---|---|---|---|---|---|---|
| `CREATED` | `run.yaml`, `run-manifest.md`, `00-input.md` | Coordinator | Confirm task record, check subagent runtime, spawn Planner | Implement code, approve contract, evaluate, emulate Planner, inspect source for direct repair | Updated `run.yaml` and `run-manifest.md` | `PLANNING`, `CANCELLED`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE` |
| `PLANNING` | `00-input.md`, project/codebase context as needed | `planner` | Analyze scope, classify run, write plan and measurable implementation contract | Implement code, approve own contract, evaluate | `01-planner-brief.md` and `02-implementation-contract.md` | `CONTRACT_REVIEW`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `CONTRACT_REVIEW` | `02-implementation-contract.md` | `contract-reviewer` | Approve or reject contract, document gaps | Implement code, rewrite contract silently | `03-contract-review.md` | `APPROVED_FOR_IMPLEMENTATION`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `APPROVED_FOR_IMPLEMENTATION` | Approved `03-contract-review.md`, `approved_for_implementation: true`, `generator_allowed: true` | Coordinator | Spawn Generator | Start evaluation, claim implementation done, generate code directly, edit source/tests/config | Updated `run.yaml` and `run-manifest.md` | `GENERATING`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `GENERATING` | Approved contract and review artifacts | `generator` | Implement only the approved contract, record commands and diff summary | Change contract scope, self-evaluate, mark complete | `04-implementation-report.md` | `EVALUATING`, `FAILED_VERIFICATION`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `EVALUATING` | `04-implementation-report.md`, code diff, verification commands | `evaluator` | Verify with real evidence, pass/fail/block | Patch implementation to make tests pass, rely on hidden memory, be same executor as Generator | `05-evaluator-report.md` | `COMPLETED`, `FAILED_VERIFICATION`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `COMPLETED` | Passing `05-evaluator-report.md` with command/evidence sections | Coordinator | Aggregate final summary from `05-evaluator-report.md`, `04-implementation-report.md`, and `run.yaml` | Claim completion without evaluator report, add new implementation evidence from memory | `06-final-summary.md` | None |
| `REJECTED_FOR_REPLAN` | Rejection in contract review or evaluation | `planner` | Revise plan/contract within run scope | Implement before reapproval | Updated `01-planner-brief.md` and/or `02-implementation-contract.md` | `CONTRACT_REVIEW`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `BLOCKED_FOR_EXECUTOR_UNAVAILABLE` | Current `run.yaml`, `run-manifest.md`, `blocked_reason` | Coordinator | Report missing subagent runtime | Use handoff files, continue required role work in same session, write role artifacts | Updated `run.yaml` and `run-manifest.md` with block reason | None until a runtime with subagent spawning is available |
| `FAILED_VERIFICATION` | Failing `05-evaluator-report.md`; bounded Generator rework packet | `generator` for fixes, then `evaluator` for recheck | Generator fixes only verified failures and documents updated implementation report; Evaluator independently rechecks | Coordinator fixes code, Coordinator adds tests, Coordinator reads source for direct repair, Evaluator patches implementation, Generator approves own fix | Updated `04-implementation-report.md` then updated `05-evaluator-report.md` | `EVALUATING`, `COMPLETED`, `REJECTED_FOR_REPLAN`, `BLOCKED_FOR_EXECUTOR_UNAVAILABLE`, `CANCELLED` |
| `CANCELLED` | Cancellation reason | Orchestrator | Record cancellation | Continue implementation or evaluation | Updated `run.yaml` and optional note in `06-final-summary.md` | None |

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

The Coordinator may update state fields after a role artifact exists. It must not invent approval, implementation, repair, or verification decisions.
