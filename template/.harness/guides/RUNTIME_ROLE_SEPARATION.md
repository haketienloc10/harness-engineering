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

New run artifacts should include runtime metadata near the top:

```yaml
runtime_mode: production_multi_session | fallback_single_session
independence: independent | degraded
role: "<Planner Agent | Contract Reviewer Agent | Generator Agent | Evaluator Agent | Coordinator>"
session: "<session id / agent id / manual label>"
```

Existing old runs may not have runtime metadata. New runs should include it. Old runs should not be rewritten unless explicitly requested.

## Fallback Mode

Single-session simulation is allowed only for local experimentation, small low-risk tasks, or environments without multi-agent support.

Fallback artifacts must include:

```yaml
runtime_mode: fallback_single_session
independence: degraded
reason: "<why separate sessions were unavailable>"
```

Fallback mode is not production-grade. A fallback Evaluator must still use visible artifacts, commands, diffs, logs, runtime checks, browser/API evidence, and acceptance criteria instead of hidden reasoning.

## Handoff Protocol

Each phase must end with a clear handoff note:

- current phase result;
- artifacts produced;
- next role allowed to proceed or blocked;
- reason if blocked.

Contract Review decision:

- `APPROVED`: Generator may start.
- `REJECTED`: Planner must revise the contract before implementation.

Evaluator decision:

- `PASS`: final summary may be produced from approved evidence.
- `FAIL` / `Blocked`: Generator or Coordinator must fix within contract or return to Planner if scope/contract is invalid.
