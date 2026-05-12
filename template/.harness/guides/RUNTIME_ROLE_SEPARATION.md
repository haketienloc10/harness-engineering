# Runtime Role Separation

## Purpose

Harness dùng các runtime role tách biệt để giảm self-approval, context contamination, và implementation bias.

## Production Mode

Production mode yêu cầu các agent/session riêng cho:

- Planner Agent
- Contract Reviewer Agent
- Generator Agent
- Evaluator Agent

Một role không được approve output của chính nó. Contract Reviewer không được là cùng runtime session đã authored contract. Evaluator không được là cùng runtime session đã generated implementation.

Production implementation tasks must use:

```yaml
runtime_mode: production_multi_session
independence: independent
```

Một runtime session không được đóng nhiều production roles trong cùng một run.

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

Each role should run in a fresh session where possible.

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
runtime_mode: production_multi_session | fallback_single_session
independence: independent | degraded
role: Planner | ContractReviewer | Generator | Evaluator | Coordinator
session_id: <manual label or runtime id>
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

Environment limitations are not a production fallback permission. If independent role sessions are unavailable for real implementation work, the current agent must stop after its assigned role and produce a handoff prompt for the next independent session.

Fallback artifacts must include:

```yaml
runtime_mode: fallback_single_session
independence: degraded
reason: "<why fallback is allowed for this task>"
```

Fallback mode is not production-grade. A fallback Evaluator must still use visible artifacts, commands, diffs, logs, runtime checks, browser/API evidence, and acceptance criteria instead of hidden reasoning.

## Blocking Rule

If a task requires production multi-session and the environment cannot spawn separate sessions, mark the run blocked for handoff:

```md
## Role Separation Status

- Production multi-session required: yes
- Current session role: <role>
- Next required role: <role>
- Same-session fallback allowed: no
- Status: BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF
- Reason: This task requires independent runtime roles.
```

## Handoff Protocol

Each phase must end with a clear handoff note:

```md
## Handoff

- Completed role:
- Artifacts produced:
- Next required role:
- Allowed next actions:
- Blocked actions:
- Notes for next role:
```

Contract Review decision:

- `APPROVED`: Generator may start.
- `REJECTED`: Planner must revise the contract before implementation.

Evaluator decision:

- `PASS`: final summary may be produced from approved evidence.
- `FAIL` / `Blocked`: Generator or Coordinator must fix within contract or return to Planner if scope/contract is invalid.
