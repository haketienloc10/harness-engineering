# Epic Lifecycle

## Required Execution Model

Epic planning and child runs use the same strict Codex project-scoped subagent orchestration rules as the default lifecycle.

Core lifecycle roles must run as separate spawned subagents from Codex project-scoped agents in `.codex/agents/`.

There is no degraded single-session fallback.

The Coordinator is orchestration-only. It must not implement, debug, repair, review, verify, approve, edit source/tests/config, or write role artifacts directly for an Epic or child run.

## Steps

1. Coordinator starts run.
2. Coordinator checks subagent runtime availability.
3. If unavailable, block run immediately.
4. If available, spawn Planner from `.codex/agents/harness-planner.toml`.
5. Planner writes `01-planner-brief.md`.
6. Planner writes both `01-planner-brief.md` and `02-implementation-contract.md` in one planning invocation, then the child run enters `CONTRACT_REVIEW`.
7. Spawn Contract Reviewer from `.codex/agents/harness-contract-reviewer.toml`.
8. Contract Reviewer writes `03-contract-review.md`.
9. If contract rejected, return to Planner/contract revision.
10. If approved, spawn Generator from `.codex/agents/harness-generator.toml`.
11. Generator writes `04-implementation-report.md`.
12. Spawn Evaluator from `.codex/agents/harness-evaluator.toml`.
13. Evaluator writes `05-evaluator-report.md`.
14. If Evaluator returns a non-passing result, Coordinator reads only the evaluator decision summary, creates a bounded Generator rework packet from `.harness/templates/generator-rework-packet.template.md`, spawns Generator, then spawns Evaluator again.
15. Run completes only if Evaluator result is `pass`.

## Epic Rule

An Epic may coordinate multiple child runs, but every child run must independently satisfy the strict lifecycle. The coordinator must not combine Planner, Contract Reviewer, Generator, and Evaluator work into one session for an Epic or child run.

Epic-level coordination may summarize child-run approved artifacts only. It must not repair a failed child run directly; it must route the child run back to Planner, Generator, or Evaluator according to the failed artifact.

## Block Rule

If subagent runtime is unavailable, create or update `run-manifest.md` with `subagent_runtime_available: false`, `run_status: blocked`, and all required role instances set to `blocked`.

If implementation or rework is required and Generator cannot be spawned, stop with `BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE`.
