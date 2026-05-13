# Harness Run Classification

Use before creating any Harness run.

## Use When

- Starting a task that might create a run.
- The task mentions phases, parts, large scope, MVP, full feature, core loop, complete playable, multiple modules, multiple user flows, or unclear verification boundary.
- A normal run contract looks broad, vague, or hard to verify as one unit.

## Load

Read only the relevant parts of:

```txt
.harness/guides/RUN_CLASSIFICATION.md
.harness/guides/LONG_TASK_POLICY.md
```

## Output

Return one decision:

- `Normal Run`
- `Epic`
- `Child Run`
- `Invalid oversized normal run`

Then take the required next action: create a bounded normal run, create an Epic, create a child run, or mark the oversized run superseded by an Epic.
