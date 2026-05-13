# Harness Lifecycle

Use before non-trivial implementation work to enforce lifecycle state, role separation, Codex project-scoped agents, and manifest audit.

## Use When

- Starting any non-trivial implementation run.
- Reviewing or approving an implementation contract.
- Routing from planning to implementation or from implementation to evaluation.
- A task might otherwise be handled by one agent/session playing multiple lifecycle roles.
- Subagent spawning is unavailable and the run must block.

## Load

Read only the relevant parts of:

```txt
.harness/guides/LIFECYCLE_ORCHESTRATION.md
.harness/guides/SUBAGENT_EXECUTION.md
.harness/workflows/default-lifecycle.md
```

## Rules

- Coordinator is orchestration-only.
- Core lifecycle work must be performed by the named Codex project-scoped agents:
  - `harness_planner`
  - `harness_contract_reviewer`
  - `harness_generator`
  - `harness_evaluator`
- `dispatch-role.sh` creates routing metadata only; Codex-native agent invocation is the execution path.
- After a subagent writes its required artifact, record completion with:
  `bash .harness/scripts/record-role-completion.sh .harness/runs/<RUN_ID> <role> <executor_id>`
- `record-role-completion.sh` is the canonical way to update `run.yaml`, `run-manifest.md`, and role artifact runtime metadata after role completion.
- No degraded single-session fallback.
- If Generator or Evaluator must re-run, route through the lifecycle instead of patching directly as Coordinator.

## Output

- Correct `run.yaml` lifecycle state.
- Correct `run-manifest.md` execution mode and role status.
- Required role artifact for the current state.
- `BLOCKED_FOR_EXECUTOR_UNAVAILABLE` when subagent spawning is unavailable.
- `BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE` when implementation or rework requires Generator but Generator cannot be spawned.
