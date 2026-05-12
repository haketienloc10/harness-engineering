# Runtime Role Separation

## Purpose

Harness uses strict template-based subagent orchestration to prevent self-approval, context contamination, and implementation bias.

The top-level agent is the coordinator/orchestrator. The coordinator routes lifecycle state, but must not execute Planner, Contract Reviewer, Generator, or Evaluator work.

## Required Lifecycle

```txt
Planner -> Contract Reviewer -> Generator -> Evaluator
```

Each core lifecycle role must be a separate spawned subagent instantiated from its fixed template:

- Planner: `.harness/subagents/planner.md`
- Contract Reviewer: `.harness/subagents/contract-reviewer.md`
- Generator: `.harness/subagents/generator.md`
- Evaluator: `.harness/subagents/evaluator.md`

The coordinator may pass task-specific inputs to the selected template, including original request, project context, relevant files, previous artifacts, acceptance criteria, verification commands, and constraints.

The coordinator must not create free-form prompts for these roles, modify role responsibilities, weaken evidence requirements, bypass role separation, or write role artifacts on behalf of subagents.

## Runtime Requirement

Core lifecycle execution requires real subagent spawning.

If the runtime cannot spawn subagents, the coordinator must block the run before Planner execution.

Required blocked message:

```text
Subagent runtime unavailable.
Harness lifecycle requires template-based subagent orchestration.
This run is blocked.
No lifecycle role may be executed in this session.
```

There is no degraded single-session fallback.

## Required Metadata

New run artifacts must include runtime metadata near the top:

```yaml
runtime_mode: template_subagents_required
executor_type: subagent
executor_id: <required>
agent_runtime: <required>
agent_session_id: <required>
role_template: .harness/subagents/<role>.md
independence: independent
role: Planner | ContractReviewer | Generator | Evaluator | Coordinator
```

Existing old runs may not have this metadata. New runs should include it. Old runs should not be rewritten unless explicitly requested.

## Role Independence Audit

A Harness run is invalid if any of the following is true:

- Planner, Contract Reviewer, Generator, and Evaluator were performed by the same session.
- The coordinator wrote lifecycle artifacts on behalf of role subagents.
- A core role was executed from a free-form prompt instead of a role template.
- The run continued after detecting unavailable subagent runtime.
- Evaluator approved without independent evidence.

## Role Matrix

| Role | May create | May read | Must not do |
|---|---|---|---|
| Planner | `01-planner-brief.md`, `02-implementation-contract.md` | user request, project context, relevant guides, relevant source/tests for planning | implement application code, approve contract, approve final evaluation |
| Contract Reviewer | `03-contract-review.md` | `00-input.md`, `01-planner-brief.md`, `02-implementation-contract.md`, project rules/verification notes | implement, rewrite contract silently, approve vague or untestable contracts |
| Generator | code changes, `04-implementation-report.md`, `06-fix-report.md` | approved contract, contract review, relevant source/tests | approve own work, broaden scope, weaken verification criteria |
| Evaluator | `05-evaluator-report.md`, maybe `07-final-summary.md` | all visible artifacts, git diff, command output, runtime/browser/API evidence, logs | implement, approve without evidence, rely on Generator statements without verification |
