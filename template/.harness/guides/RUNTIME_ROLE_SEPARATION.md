# Runtime Role Separation

## Purpose

Harness is agent-runtime agnostic.

Harness dùng các runtime role tách biệt để giảm self-approval, context contamination, và implementation bias.

Top-level agent là Orchestrator. Orchestrator điều phối lifecycle state, nhưng không làm thay role production khi current agent runtime hỗ trợ independent executor.

## Production Mode

Production mode yêu cầu independent role executors cho:

- Planner Agent
- Contract Reviewer Agent
- Generator Agent
- Evaluator Agent

Nếu current agent runtime hỗ trợ subagent, task tool, external agent session, isolated role executor, hoặc role-specific process, role-specific execution là mandatory.

Một role không được approve output của chính nó. Contract Reviewer không được là cùng executor đã authored contract. Evaluator không được là cùng executor đã generated implementation.

Production implementation tasks must use:

```yaml
runtime_mode: production_multi_executor
executor_type: subagent | task_tool | external_agent_session | isolated_process | fallback_single_session
executor_id: <required>
agent_runtime: <required>
agent_session_id: <required>
independence: independent
```

Một executor không được đóng nhiều production roles trong cùng một run.

Lifecycle chuẩn:

```txt
User Request
  -> Planner Agent
  -> Contract Reviewer Agent
  -> Generator Agent
  -> Evaluator Agent
  -> Final Summary
```

## Role Matrix

| Role | May create | May read | Must not do |
|---|---|---|---|
| Planner Agent | `00-input.md`, `01-planner-brief.md`, `02-implementation-contract.md` | user request, project context, relevant guides, relevant source/tests for planning | implement application code, approve contract, approve final evaluation |
| Contract Reviewer Agent | `03-evaluator-contract-review.md` | `00-input.md`, `01-planner-brief.md`, `02-implementation-contract.md`, project rules/verification notes | implement, rewrite contract silently, approve vague or untestable contracts |
| Generator Agent | code changes, `04-generator-worklog.md`, `06-fix-report.md` | approved contract, contract review, relevant source/tests | approve own work, broaden scope, weaken verification criteria |
| Evaluator Agent | `05-evaluator-report.md`, maybe `07-final-summary.md` | all visible artifacts, git diff, command output, runtime/browser/API evidence, logs | implement, approve without evidence, rely on Generator statements without verification |

## Context Isolation

Each role should run in an independent executor where possible. When the current runtime supports independent executors, this is required.

Evaluator must not use Planner/Generator hidden reasoning or memory. Evaluator must cite visible evidence from artifacts, diffs, command output, runtime checks, browser/API evidence, screenshots descriptions, or logs.

Contract Reviewer must review only visible inputs and must reject contracts that require hidden Planner assumptions to understand.

## Allowed Inputs By Phase

Planner Agent may use:

- user request;
- `.harness/project/*` adapter files;
- relevant guides;
- source/test inspection needed for planning.

Contract Reviewer Agent may use:

- `00-input.md`;
- `01-planner-brief.md`;
- `02-implementation-contract.md`;
- project rules and verification notes if needed.

Generator Agent may use:

- approved `02-implementation-contract.md`;
- `03-evaluator-contract-review.md`;
- relevant source code and tests.

Evaluator Agent may use:

- original input;
- planner brief;
- approved contract;
- contract review;
- generator worklog;
- git diff;
- verification commands and outputs;
- runtime/UI/API evidence when relevant.

## Required Metadata

New run artifacts must include runtime metadata near the top:

```yaml
runtime_mode: production_multi_executor | fallback_single_session
executor_type: subagent | task_tool | external_agent_session | isolated_process | fallback_single_session
executor_id: <required>
agent_runtime: <required>
agent_session_id: <required>
independence: independent | degraded
role: Planner | ContractReviewer | Generator | Evaluator | Coordinator
```

Existing old runs may not have runtime metadata. New runs should include it. Old runs should not be rewritten unless explicitly requested.

## Fallback Mode

Fallback single-session mode is not production-grade.

It is allowed only for:

- local experimentation;
- low-risk documentation-only tasks;
- learning/demo workflows;
- tasks explicitly marked by the user as fallback-allowed.

Fallback mode is forbidden for:

- multi-phase tasks;
- Epic tasks;
- child runs inside Epic;
- implementation tasks affecting application behaviour;
- UI/API behaviour implementation;
- production-grade workflow;
- tasks requiring independent contract review or independent evaluation;
- tasks where the user explicitly requires independent roles.

Environment limitations are not a production fallback permission. If independent role executors are unavailable for real implementation work, the current agent must stop at the role boundary and set `BLOCKED_FOR_EXECUTOR_UNAVAILABLE` unless `fallback_single_session_allowed: true` is explicitly set in `run.yaml`.

Fallback artifacts must include:

```yaml
runtime_mode: fallback_single_session
independence: degraded
reason: "<why fallback is allowed for this task>"
```

Fallback mode is not production-grade. A fallback Evaluator must still use visible artifacts, commands, diffs, logs, runtime checks, browser/API evidence, and acceptance criteria instead of hidden reasoning.

## Blocking Rule

If a task requires production multi-executor and the environment cannot spawn independent executors, mark the run blocked:

```md
## Role Separation Status

- Production multi-executor required: yes
- Current session role: <role>
- Next required role: <role>
- Same-session fallback allowed: no
- Status: BLOCKED_FOR_EXECUTOR_UNAVAILABLE
- Reason: This task requires independent runtime roles.
```

Do not create `HANDOFF.md` for role transitions.

## Next Role Note

Each phase may end with a clear Next role note for traceability:

```md
## Next Role

- Completed role:
- Artifacts produced:
- Next required role:
- Allowed next actions:
- Blocked actions:
- Notes for next role:
```

This note is not a handoff file and must not replace executor dispatch.

Contract Review decision:

- `APPROVED`: Generator may start.
- `REJECTED`: Planner must revise the contract before implementation.

Evaluator decision:

- `PASS`: final summary may be produced from approved evidence.
- `FAIL` / `Blocked`: Generator or Coordinator must fix within contract or return to Planner if scope/contract is invalid.
