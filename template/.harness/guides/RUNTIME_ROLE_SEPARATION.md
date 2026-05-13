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

## Coordinator Non-Execution Policy

The coordinator is an orchestration role only.

The coordinator MUST NOT perform implementation, review, verification, debugging, or repair work directly.

Forbidden coordinator actions:

- editing application source files;
- editing tests;
- editing production configuration;
- applying patches;
- writing production code;
- fixing implementation bugs directly;
- running exploratory source-code modification loops;
- reading implementation files for the purpose of direct repair;
- acting as Planner, Contract Reviewer, Generator, or Evaluator;
- approving its own work;
- producing implementation or evaluation evidence without the corresponding role artifact.

Allowed coordinator actions:

- classify the request;
- create or update run/epic structure;
- build bounded role packets;
- spawn the required template-based subagent;
- wait for role completion;
- inspect role status, decision summaries, and required role artifacts;
- update Harness orchestration status;
- route rejected or failed artifacts back to the correct role;
- stop the workflow when required role execution is unavailable;
- summarize final result from approved artifacts only.

## Coordinator Source Edit Ban

The coordinator MUST NOT modify application source, tests, runtime configuration, generated production artifacts, or project implementation files.

The coordinator may modify only Harness orchestration artifacts, such as:

- run status files;
- routing notes;
- rework packets;
- role packets;
- final orchestration summaries;
- Harness metadata files.

Any implementation change MUST be performed by the Generator role.

If a source/test/config change is required, the coordinator MUST spawn the Generator role.

If the runtime cannot spawn the Generator role, stop with:

```text
BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
```

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

If a required role template is missing, invalid, or unavailable, stop with:

```text
BLOCKED_REQUIRED_SUBAGENT_TEMPLATE_UNAVAILABLE
```

If the runtime cannot spawn required subagents, stop with:

```text
BLOCKED_REQUIRED_SUBAGENT_UNAVAILABLE
```

## Required Metadata

New run artifacts must include runtime metadata near the top:

```yaml
runtime_mode: template_subagents_required
executor_type: subagent
executor_id: <required>
agent_runtime: <required>
agent_session_id: <required>
template_source: .harness/subagents/<role>.md
started_at: <required>
completed_at: <required>
independence: independent
role: Planner | ContractReviewer | Generator | Evaluator | Coordinator
```

New role artifacts must use `template_source` for validator checks. `role_template` may exist only as legacy context.

Existing old runs may not have this metadata. New runs should include it. Old runs should not be rewritten unless explicitly requested.

## Role Independence Audit

A Harness run is invalid if any of the following is true:

- Planner, Contract Reviewer, Generator, and Evaluator were performed by the same session.
- The coordinator wrote lifecycle artifacts on behalf of role subagents.
- A core role was executed from a free-form prompt instead of a role template.
- The run continued after detecting unavailable subagent runtime.
- Evaluator approved without independent evidence.
- The coordinator modified source, tests, production config, or implementation artifacts.
- The coordinator repaired rejected work directly instead of routing to the responsible role.

## Artifact-Only Role Inputs

Roles communicate through written artifacts, not inherited raw conversation history.

A downstream role may read only the approved upstream artifact and the explicitly allowed input artifacts.

Forbidden transcript transfer:

- passing full Planner transcript to Contract Reviewer;
- passing full Reviewer transcript to Generator;
- passing full Generator transcript to Evaluator;
- passing unrelated previous run artifacts by default;
- coordinator summarizing raw implementation details from memory instead of using artifacts.

Allowed artifact input chain:

- Planner -> planner brief and contract;
- Contract Reviewer -> contract review decision;
- Generator -> implementation report, changed files summary, verification commands/results;
- Evaluator -> evaluator report and pass/fail decision.

## Coordinator Context Budget Guard

Coordinator calls SHOULD remain small after role delegation.

Expected coordinator input budget:

- normal orchestration step: <= 20,000 input tokens;
- routing/rework step: <= 30,000 input tokens;
- exceptional recovery step: <= 40,000 input tokens.

If a coordinator step exceeds 80,000 input tokens after a subagent has completed, treat this as a context packaging bug.

The coordinator must stop and emit:

```text
BLOCKED_COORDINATOR_CONTEXT_OVER_BUDGET
```

Required recovery:

- do not continue implementation in coordinator context;
- compact current state into a bounded role packet;
- spawn the next required role.

## Role Matrix

| Role | May create | May read | Must not do |
|---|---|---|---|
| Planner | `01-planner-brief.md`, `02-implementation-contract.md` | user request, project context, relevant guides, relevant source/tests for planning | implement application code, approve contract, approve final evaluation |
| Contract Reviewer | `03-contract-review.md` | `00-input.md`, `01-planner-brief.md`, `02-implementation-contract.md`, project rules/verification notes | implement, rewrite contract silently, approve vague or untestable contracts |
| Generator | code changes, `04-implementation-report.md` | approved contract, contract review, relevant source/tests | approve own work, broaden scope, weaken verification criteria |
| Evaluator | `05-evaluator-report.md`, maybe `06-final-summary.md` | all visible artifacts, git diff, command output, runtime/browser/API evidence, logs | implement, approve without evidence, rely on Generator statements without verification |
| Coordinator | orchestration status, routing notes, role/rework packets, final summary from approved artifacts | run state, manifests, role status, decision summaries, approved artifacts | source edits, test edits, production config edits, debugging, implementation repair, role artifact authorship |
