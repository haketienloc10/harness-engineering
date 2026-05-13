# Harness Epic

Use for long-running or multi-phase tasks that must be decomposed into independently verifiable child runs.

## Use When

- The task has multiple phases, milestones, modules, user flows, or verification checkpoints.
- The task text mentions `phase`, `part`, `core loop`, `complete playable`, `full feature`, `end-to-end`, `MVP`, `large task`, or `long task`.
- A created normal run is discovered to be oversized.

## Load

Read only the relevant parts of:

```txt
.harness/guides/LONG_TASK_POLICY.md
.harness/workflows/epic-lifecycle.md
```

## Output

- Epic container under `.harness/runs/EPIC-*`.
- Child-run plan with at least two independently verifiable child runs.
- No implementation inside an oversized normal run.
