# AGENTS.md

## Purpose

This repository uses **Harness** to make AI-assisted development auditable and verifiable.

This root file is a **bootstrap instruction**. Keep it small. It defines mandatory rules that apply to every task. Detailed procedures live in `.harness/guides/*` and should be loaded only when relevant.

The user should be able to give a normal development request. The agent must orchestrate Planner, Generator, and Evaluator internally.

---

## Repository Boundary

The `.harness/` tree contains workflow guides, templates, scripts, run records, and backlog items for AI-assisted development.

It is not the application source tree. When Harness is installed into a real project, agents must treat `.harness/` as workflow infrastructure and inspect the host project separately for application code, tests, runtime behaviour, and architecture.

---

## Priority Order

When instructions conflict, follow this order:

1. The current user request.
2. Root `AGENTS.md`.
3. Relevant files in `.harness/guides/*`.
4. Templates in `.harness/templates/*`.
5. Agent defaults or assumptions.

Generated Harness artifacts must be written in Vietnamese unless the content is a technical identifier, command, path, code, config key, log, error message, API field, schema key, package name, or copied tool output.

---

## Mandatory Harness Lifecycle

For every non-trivial implementation task, create one run under:

```txt
.harness/runs/RUN-YYYYMMDD-NNN-task-slug/
```

A valid run must contain and maintain:

```txt
run.yaml
00-input.md
01-planner-brief.md
02-implementation-contract.md
03-evaluator-contract-review.md
04-generator-worklog.md
05-evaluator-report.md
07-final-summary.md
```

If evaluation fails and fixes are needed, also write:

```txt
06-fix-report.md
```

Always update:

```txt
.harness/runs/RUN_INDEX.md
```

Do not modify application code until `03-evaluator-contract-review.md` approves `02-implementation-contract.md`.

---

## Required Role Separation

The agent may perform Planner, Generator, and Evaluator in one conversation turn, but their artifacts must remain separate.

- Planner defines goal, scope, non-scope, acceptance criteria, likely impacted areas, risks, and unknowns.
- Generator implements only the approved contract.
- Evaluator verifies against the original input, planner brief, contract, acceptance criteria, and real evidence.

Evaluator must not approve by code inspection alone.

---

## Verification Requirements

Run real verification whenever possible.

Default:

```bash
bash .harness/scripts/verify.sh
```

If the app has runtime UI or API behaviour, also run smoke/runtime checks:

```bash
bash .harness/scripts/smoke.sh
```

For Vite apps:

```bash
APP_URL=http://localhost:5173 bash .harness/scripts/smoke.sh
```

For UI tasks, build success, static checks, or curl smoke are not enough. Evaluator must provide behaviour-level evidence for each required UI behaviour such as validation, create/update/delete, filtering, navigation, state transition, persistence, error state, and empty state.

If required behaviour evidence is missing, the run must be marked `Fail`, `Needs Fix`, or `Blocked`, not `Pass`.

---

## Code Change Rules

Before editing files:

- read the target file first
- inspect nearby code
- search usages before changing existing functions/classes
- avoid unrelated refactors
- keep changes within the approved contract

Do not edit Harness guides, templates, or scripts unless the user explicitly asks. If a run reveals a reusable Harness improvement, add a concrete proposal to:

```txt
.harness/backlog/HARNESS_BACKLOG.md
```

---

## Parallel Work

If the user gives multiple unrelated tasks, create one run per task.

Before implementation, check active runs for file conflicts. If runs may modify the same file, record the conflict and prefer separate branches or worktrees. Do not proceed silently.

Recommended branch:

```txt
feat/RUN-YYYYMMDD-NNN-task-slug
```

Recommended worktree:

```txt
../worktrees/RUN-YYYYMMDD-NNN-task-slug
```

---

## When To Read Detailed Guides

Load only the guide needed for the task:

```txt
.harness/guides/AGENTS.md
.harness/guides/LANGUAGE_POLICY.md
.harness/guides/PLANNING_AND_CONTRACTS.md
.harness/guides/TESTING_POLICY.md
.harness/guides/PARALLEL_WORK.md
.harness/guides/BACKLOG_POLICY.md
```

Do not load every guide by default.
