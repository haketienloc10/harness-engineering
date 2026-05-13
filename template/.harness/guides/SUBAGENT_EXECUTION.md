# Subagent Execution

Harness core lifecycle execution requires Codex project-scoped subagents.

Role transitions are performed by invoking the named Codex project-scoped agent. `.harness/scripts/dispatch-role.sh` may be used first to create deterministic routing metadata.

Harness does not use `HANDOFF.md` for normal lifecycle transitions.

`dispatch-role.sh` creates a dispatch artifact only:

```txt
.harness/runs/<RUN_ID>/dispatch/<role>.dispatch.md
```

It does not spawn, execute, or emulate a subagent. Codex-native invocation of the named project-scoped agent is the canonical execution path.

## Runtime Capability Rule

Before executing lifecycle role work, determine whether the current runtime supports real subagent spawning.

New runs start with `runtime.subagent_runtime_available: unknown` in `run.yaml` and `subagent_runtime_available: unknown` in `run-manifest.md`.

If supported, the coordinator MUST mark runtime availability as true before Planner dispatch, then call `.harness/scripts/dispatch-role.sh` for routing metadata and invoke the corresponding named Codex agent in `.codex/agents/`.

If unsupported, the coordinator MUST mark runtime availability as false and block the run before Planner execution.

There is no degraded single-session fallback.

Use the helper when available:

```bash
bash .harness/scripts/set-runtime-capability.sh .harness/runs/<RUN_ID> true
bash .harness/scripts/set-runtime-capability.sh .harness/runs/<RUN_ID> false "Subagent runtime unavailable"
```

This helper records a manual assertion, not a runtime proof. If the current agent/runtime cannot spawn named Codex project-scoped agents, Harness lifecycle execution is blocked.

If a required Codex agent file is missing, invalid, or unavailable, the coordinator MUST stop with:

```text
BLOCKED_REQUIRED_CODEX_AGENT_UNAVAILABLE
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
Harness lifecycle requires Codex project-scoped subagents from `.codex/agents/`.
This run is blocked.
No lifecycle role may be executed in this session.
```

## Required Codex Agents

- `harness_planner` -> `.codex/agents/harness-planner.toml`
- `harness_contract_reviewer` -> `.codex/agents/harness-contract-reviewer.toml`
- `harness_generator` -> `.codex/agents/harness-generator.toml`
- `harness_evaluator` -> `.codex/agents/harness-evaluator.toml`

## Hard Rules

1. The Orchestrator controls lifecycle routing and state transitions only.
2. The Orchestrator MUST NOT perform required role work directly.
3. The Orchestrator MUST NOT create `HANDOFF.md` for normal role transitions.
4. The Orchestrator MUST invoke the next named Codex project-scoped subagent.
5. `run.yaml` is the authoritative lifecycle state source.
6. Contract Reviewer must not implement code.
7. Generator must not evaluate its own output.
8. Evaluator must not patch implementation to make tests pass.
9. Evaluator must be independent from Generator.
10. Contract Reviewer must be independent from Planner.
11. Single-agent role execution is invalid.
12. If subagent spawning is unavailable, set the run state to `BLOCKED_FOR_EXECUTOR_UNAVAILABLE` and update `run-manifest.md`.
13. The Orchestrator MUST NOT create free-form prompts for core lifecycle roles.
14. The Orchestrator MUST NOT modify Codex agent responsibilities, forbidden actions, required artifacts, evidence requirements, or pass/fail criteria.
15. The Orchestrator MUST NOT edit application source, tests, production configuration, generated production artifacts, or project implementation files.
16. The Orchestrator MUST NOT debug or repair implementation failures directly.
17. The Orchestrator MUST NOT read implementation files for the purpose of direct repair after a role returns.
18. The Orchestrator MUST route failed or rejected work back to the responsible Codex project-scoped role.
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
| `PLANNING` | `planner` | `01-planner-brief.md` and `02-implementation-contract.md` |
| `CONTRACT_REVIEW` | `contract-reviewer` | `03-contract-review.md` |
| `GENERATING` | `generator` | code changes + `04-implementation-report.md` |
| `EVALUATING` | `evaluator` | `05-evaluator-report.md` |
| `FAILED_VERIFICATION` | `generator`, then `evaluator` | updated `04-implementation-report.md`, then updated `05-evaluator-report.md` |

## Evaluator Failure Routing

When Evaluator returns `FAIL`, `REJECTED`, `NEEDS_FIX`, `blocked_insufficient_evidence`, or any equivalent non-passing result, the Orchestrator must:

1. Read the evaluator decision summary only.
2. Create a bounded Generator rework packet from `.harness/templates/generator-rework-packet.template.md`.
3. Invoke `harness_generator`.
4. Wait for an updated `04-implementation-report.md`.
5. Invoke `harness_evaluator` again.

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

## Role Codex Agent Files

Record subagent metadata in `run.yaml`, `run-manifest.md`, and role artifacts.

Allowed executor type:

- `subagent`

No other executor type is valid for core lifecycle roles.

Use the helper after a role subagent finishes:

```bash
bash .harness/scripts/record-role-completion.sh .harness/runs/<RUN_ID> <role> <executor_id>
```

## Removed Handoff Behavior

Do not create `HANDOFF.md`.

Do not ask the user to manually continue the next role if an executor can be started.

Do not use forbidden handoff files as phase boundaries.

Lifecycle role boundaries are enforced by spawned subagents from Codex project-scoped agents and artifact ownership.
