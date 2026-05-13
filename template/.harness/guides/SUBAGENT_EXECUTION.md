# Subagent Execution

Harness core lifecycle execution requires template-based spawned subagents.

Role transitions are performed through `.harness/scripts/dispatch-role.sh`, then by spawning independent subagents from fixed templates.

Harness does not use `HANDOFF.md` for normal lifecycle transitions.

## Runtime Capability Rule

Before executing lifecycle role work, determine whether the current runtime supports real subagent spawning.

If supported, the coordinator MUST call `.harness/scripts/dispatch-role.sh` for the next lifecycle role. The runtime executor MUST instantiate that role from the corresponding template under `.harness/subagents/`.

If unsupported, the coordinator MUST block the run before Planner execution.

There is no degraded single-session fallback.

If a required role template is missing, invalid, or unavailable, the coordinator MUST stop with:

```text
BLOCKED_REQUIRED_SUBAGENT_TEMPLATE_UNAVAILABLE
```

If a required subagent cannot be spawned, the coordinator MUST stop with:

```text
BLOCKED_REQUIRED_SUBAGENT_UNAVAILABLE
```

If the missing role is Generator and implementation or rework is required, the coordinator MUST stop with:

```text
BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
```

Required blocked message:

```text
Subagent runtime unavailable.
Harness lifecycle requires template-based subagent orchestration.
This run is blocked.
No lifecycle role may be executed in this session.
```

## Required Templates

- `.harness/subagents/planner.md`
- `.harness/subagents/contract-reviewer.md`
- `.harness/subagents/generator.md`
- `.harness/subagents/evaluator.md`

## Hard Rules

1. The Orchestrator controls lifecycle routing and state transitions only.
2. The Orchestrator MUST NOT perform required role work directly.
3. The Orchestrator MUST NOT create `HANDOFF.md` for normal role transitions.
4. The Orchestrator MUST spawn the next role-specific subagent from its template.
5. `run.yaml` is the authoritative lifecycle state source.
6. Contract Reviewer must not implement code.
7. Generator must not evaluate its own output.
8. Evaluator must not patch implementation to make tests pass.
9. Evaluator must be independent from Generator.
10. Contract Reviewer must be independent from Planner.
11. Single-agent role execution is invalid.
12. If subagent spawning is unavailable, set the run state to `BLOCKED_FOR_EXECUTOR_UNAVAILABLE` and update `run-manifest.md`.
13. The Orchestrator MUST NOT create free-form prompts for core lifecycle roles.
14. The Orchestrator MUST NOT modify template responsibilities, forbidden actions, required artifacts, evidence requirements, or pass/fail criteria.
15. The Orchestrator MUST NOT edit application source, tests, production configuration, generated production artifacts, or project implementation files.
16. The Orchestrator MUST NOT debug or repair implementation failures directly.
17. The Orchestrator MUST NOT read implementation files for the purpose of direct repair after a role returns.
18. The Orchestrator MUST route failed or rejected work back to the responsible template-based role.
19. The Orchestrator MUST use `.harness/scripts/dispatch-role.sh`; free-form role prompts are forbidden.

## Orchestrator Duties

- Read `run.yaml`.
- Read `run-manifest.md`.
- Determine the next required lifecycle state.
- Determine the next required role.
- Call `.harness/scripts/dispatch-role.sh` for the correct role.
- Ensure the runtime consumes the dispatch artifact and spawns the correct role subagent.
- Provide only the required visible inputs for that role.
- Refuse invalid transitions.
- Update `run.yaml` only after the required artifact exists.
- Update `run-manifest.md` for role instance status and runtime availability.
- Never invent role approval, implementation, or verification decisions.
- Read only role status, decision summaries, and approved artifacts when routing after delegation.
- Create bounded role/rework packets when a role must repeat work.

## Coordinator Write Scope Validator

When the current executor role is `coordinator` or `orchestrator`, run:

```bash
HARNESS_EXECUTOR_ROLE=coordinator \
HARNESS_RUN_DIR=".harness/runs/<RUN_ID>" \
bash .harness/scripts/validate-coordinator-write-scope.sh
```

The validator enforces Direction A: the coordinator may write only narrow Harness orchestration metadata in the current run. If it fails, stop with `BLOCKED_COORDINATOR_WRITE_SCOPE_VIOLATION` and route required source/test/config changes to Generator.

Coordinator MAY write only:

```text
run.yaml
run-manifest.md
routing-note.md
rework-packet.md
generator-rework-packet.md
06-final-summary.md
status.md
routing/*.md
packets/*.md
```

Coordinator MUST NOT write:

```text
01-planner-brief.md
02-implementation-contract.md
03-contract-review.md
04-implementation-report.md
05-evaluator-report.md
source code
tests
configs
package/build files
```

The coordinator may summarize final results only from approved artifacts and evaluator evidence. It must not invent implementation or verification evidence.

## Role Dispatch Table

| Lifecycle State | Required Executor | Required Output |
|---|---|---|
| `PLANNING` | `planner` | `01-planner-brief.md` |
| `CONTRACTING` | `planner` | `02-implementation-contract.md` |
| `CONTRACT_REVIEW` | `contract-reviewer` | `03-contract-review.md` |
| `GENERATING` | `generator` | code changes + `04-implementation-report.md` |
| `EVALUATING` | `evaluator` | `05-evaluator-report.md` |
| `FAILED_VERIFICATION` | `generator`, then `evaluator` | updated `04-implementation-report.md`, then updated `05-evaluator-report.md` |

## Evaluator Failure Routing

When Evaluator returns `FAIL`, `REJECTED`, `NEEDS_FIX`, `blocked_insufficient_evidence`, or any equivalent non-passing result, the Orchestrator must:

1. Read the evaluator decision summary only.
2. Create a bounded Generator rework packet from `.harness/templates/generator-rework-packet.template.md`.
3. Spawn Generator from `.harness/subagents/generator.md`.
4. Wait for an updated `04-implementation-report.md`.
5. Spawn Evaluator from `.harness/subagents/evaluator.md` again.

The Orchestrator must not edit source, inspect implementation files for direct repair, add tests directly, run a fix loop, or write role artifacts.

If Generator cannot be spawned, stop with `BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE`.

## Artifact-Only Role Inputs

Roles communicate through written artifacts, not inherited raw conversation history.

Allowed artifact input chain:

- Planner -> `01-planner-brief.md` and `02-implementation-contract.md`;
- Contract Reviewer -> `03-contract-review.md`;
- Generator -> `04-implementation-report.md`, changed files summary, and verification commands/results;
- Evaluator -> `05-evaluator-report.md` and pass/fail decision.

Do not pass full role transcripts, unrelated previous run artifacts, or coordinator memory as role inputs unless an approved artifact explicitly authorizes that input.

## Role Template Sources

Record subagent metadata in `run.yaml`, `run-manifest.md`, and role artifacts.

Allowed executor type:

- `subagent`

No other executor type is valid for core lifecycle roles.

## Removed Handoff Behavior

Do not create `HANDOFF.md`.

Do not ask the user to manually continue the next role if an executor can be started.

Do not use forbidden handoff files as phase boundaries.

Lifecycle role boundaries are enforced by spawned subagents from fixed templates and artifact ownership.
