# Harness Agent Operating Guide

## Core Operating Mode

The user provides normal development requests. The agent manages the Harness workflow automatically.

Do not ask the user to manually run Planner, Generator, or Evaluator prompts.

For every non-trivial implementation task, run the full Harness lifecycle unless the user explicitly asks for analysis only.

## Run Creation

When creating a Harness run, agents MUST use the provided script instead of creating `.harness/runs/` folders manually:

```bash
.harness/scripts/new-run.sh "task slug"
```

The script creates the next run folder using this format:

```txt
.harness/runs/RUN-YYYYMMDD-NNN-task-slug/
```

The script also copies relevant templates from `.harness/templates/`, creates/updates `run.yaml`, updates `.harness/runs/RUN_INDEX.md`, and ensures `.harness/runs/` is ignored by git.

## Required Artifacts

Each run should normally include:

```txt
00-input.md
01-planner-brief.md
02-implementation-contract.md
03-evaluator-contract-review.md
04-generator-worklog.md
05-evaluator-report.md
07-final-summary.md
```

Use `06-fix-report.md` only when evaluation fails and a fix pass is needed.

## Lifecycle Order

1. Record user input.
2. Write planner brief.
3. Write implementation contract.
4. Check conflict with active runs.
5. Evaluate the contract.
6. Implement only after contract approval.
7. Record generator worklog.
8. Run verification.
9. Write evaluator report.
10. Fix only evaluator-reported issues if needed.
11. Re-run verification.
12. Write final summary.
13. Update run status and index.
14. Add backlog proposal only if the run reveals a reusable Harness improvement.
