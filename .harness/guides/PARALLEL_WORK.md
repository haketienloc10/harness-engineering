# Parallel Work Policy

## One Task, One Run

If the user gives multiple unrelated tasks, create one Harness run per task.

Do not combine unrelated tasks into one run.

Each run must have its own:

- run folder
- `run.yaml`
- input
- planner brief
- implementation contract
- evaluator report
- final summary

## Conflict Check

Before implementation, check active runs for file conflicts.

If two active runs may modify the same file:

- record the conflict in the contract or contract review
- prefer separate branches or worktrees
- do not proceed silently

Recommended branch:

```txt
feat/RUN-YYYYMMDD-NNN-task-slug
```

Recommended worktree:

```txt
../worktrees/RUN-YYYYMMDD-NNN-task-slug
```
