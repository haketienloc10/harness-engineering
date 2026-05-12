# Harness Workflow Skills

Agents should read this registry first.

Select a relevant Harness workflow skill by name, description, and trigger conditions.

Load only the selected skill file.

Do not load all skill files by default.

## run-classification

Description:
Use before creating any run. Determines whether the request should be a normal run or an Epic.

Use when:
- Starting any task that might create a run.
- Task text mentions phases, parts, large scope, MVP, full feature, core loop, complete playable, multiple modules, multiple user flows, or unclear verification boundary.
- A normal run contract looks broad, vague, or hard to verify as one unit.

Load:
`.harness/guides/RUN_CLASSIFICATION.md`

Outputs:
- Decision: Normal Run | Epic | Child Run | Invalid oversized normal run
- Required next action: create bounded normal run, create Epic, create child run, or mark `SUPERSEDED_BY_EPIC`

## epic-workflow

Description:
Use for long-running or multi-phase tasks. Creates an Epic container and decomposes work into independently verifiable child runs.

Use when:
- Task has multiple phases, milestones, modules, user flows, or verification checkpoints.
- Task text mentions `phase 1-4`, `part 1-4`, `core loop`, `complete playable`, `full feature`, `end-to-end`, `MVP`, `large task`, or `long task`.
- A created normal run is discovered to be oversized.

Load:
`.harness/guides/LONG_TASK_POLICY.md`

Outputs:
- Epic container under `.harness/runs/EPIC-*`
- Child-run plan with at least two independently verifiable child runs
- No implementation in oversized normal runs

## project-sync

Description:
Inspect current Harness context and the host project, then create or update `.harness/project/*` with the latest evidence.

Use when:
- Harness was just installed.
- User asks to contextualize, refresh, correct, or update project context.
- `.harness/project/*` is missing, stale, contradictory, or low-confidence.
- Before a major run/epic if project context is not reliable.
- After a run/epic if new project facts were discovered.

Load:
`.harness/skills/project-sync.md`

Outputs:
- Updated `.harness/project/*`
- Summary of facts updated
- Open questions if evidence is insufficient

## codebase-sync

Description:
Create or update `.harness/codebase/*` as a lightweight source-navigation and change-impact map for future coding runs.

Use when:
- `.harness/codebase/*` is missing, stale, contradictory, or low-confidence.
- Before a non-trivial coding run when source areas, entrypoints, flows, or impact risks are unclear.
- After a run discovers durable source-navigation or change-impact facts.
- Source files, routes, commands, jobs, tests, or module boundaries moved.

Hard boundary:
- Do not duplicate `.harness/project/*`.
- Reference `.harness/project/PROJECT_PROFILE.md` for stack/runtime/package manager.
- Reference `.harness/project/PROJECT_VERIFICATION.md` for general verification.
- Keep `.harness/codebase/*` limited to source-navigation, concrete entrypoints, technical flows, source areas, change impact, source evidence, and freshness metadata.

Load:
`.harness/skills/codebase-sync.md`

Outputs:
- Updated `.harness/codebase/*`
- Summary of source areas, entrypoints, flows, and impact risks updated
- Stale or uncertain source areas if evidence is insufficient

## lifecycle-orchestration

Description:
Use the Lifecycle Orchestration and Subagent Execution guides before non-trivial implementation to enforce run states, gates, role executors, independent review/evaluation, and handoff fallback.

Use when:
- Starting any non-trivial implementation run.
- Reviewing or approving an implementation contract.
- Handing work from planning to implementation or from implementation to evaluation.
- A task might otherwise be handled by one agent/session playing multiple roles.
- Independent sessions are unavailable and a role-boundary handoff is required.

Load:
`.harness/guides/LIFECYCLE_ORCHESTRATION.md`

If Codex subagents are available, also load:
`.harness/guides/SUBAGENT_EXECUTION.md`

Outputs:
- Correct `run.yaml` lifecycle state
- Required role artifact for the current state
- `HANDOFF.md` when the next independent executor cannot run
- `BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF` when production work cannot continue in the same session
