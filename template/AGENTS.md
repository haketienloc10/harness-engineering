# AGENTS.md

Repository này đã cài **Harness** để điều phối AI-assisted development bằng artifacts, lifecycle state, Codex project-scoped agents, và workflow skills.

File này là bootstrap instruction của target repository. Chỉ load tài liệu liên quan đến task hiện tại.

## Bootstrap Rules

- Reply theo ngôn ngữ người dùng; giữ code, command, path, API name, logs, schema keys, package names, và identifiers ở dạng gốc.
- Trước non-trivial work, chọn skill liên quan trong `.codex/skills/harness-*/SKILL.md`; không load toàn bộ skills/guides/codebase cache theo mặc định.
- Dùng `run.yaml` làm authoritative lifecycle state.
- Không sửa application code trước khi Contract Reviewer approve contract và `run.yaml` cho phép Generator.
- Evaluator phải độc lập với Generator và phải có evidence thật.
- Không kết thúc bằng generic next steps nếu bước tiếp theo có thể biểu diễn bằng Harness lifecycle routing.

## Repository Boundary

`.harness/` là workflow infrastructure của target repository. Nó không phải application source tree.

Lifecycle role definitions canonical nằm trong `.codex/agents/*.toml`. Workflow skills canonical nằm trong `.codex/skills/harness-*/SKILL.md`.

## Context Loading

Trước implementation work không tầm thường:

1. Chọn skill liên quan trong `.codex/skills/harness-*/SKILL.md`.
2. Nếu cần project-level context, đọc các file liên quan trong `.harness/project/*`.
3. Nếu cần source-navigation hoặc impact context, đọc `.harness/codebase/CODEBASE_INDEX.md` và chỉ các docs liên quan.
4. Nếu project/codebase context thiếu, stale, contradictory, hoặc low-confidence, dùng `harness-project-sync` hoặc `harness-codebase-sync`.

## Harness Lifecycle

Trước khi tạo run, classify request:

```txt
User request
  -> Normal Run nếu bounded và verify được trong một run
  -> Epic nếu broad, multi-phase, multi-module, long-running, hoặc không verify sạch trong một run
```

Canonical run artifacts:

```txt
run.yaml
run-manifest.md
00-input.md
01-planner-brief.md
02-implementation-contract.md
03-contract-review.md
04-implementation-report.md
05-evaluator-report.md
06-final-summary.md
```

Normal runs live under `.harness/runs/RUN-YYYYMMDD-NNN-task-slug/`. Epic containers live under `.harness/runs/EPIC-YYYYMMDD-NNN-task-slug/`. Child runs live under `.harness/runs/EPIC-YYYYMMDD-NNN-task-slug/runs/RUN-NNN-child-task-slug/`.

Detailed classification and Epic rules live in:

```txt
.codex/skills/harness-run-classification/SKILL.md
.codex/skills/harness-epic/SKILL.md
```

## Codex Project-Scoped Agents

Harness core lifecycle roles MUST be executed by separate spawned Codex project-scoped agents:

```txt
harness_planner
harness_contract_reviewer
harness_generator
harness_evaluator
```

Required agent files:

```txt
.codex/agents/harness-planner.toml
.codex/agents/harness-contract-reviewer.toml
.codex/agents/harness-generator.toml
.codex/agents/harness-evaluator.toml
```

Coordinator is orchestration-only. Coordinator must not perform lifecycle role work, write role artifacts for subagents, edit source/tests/config for implementation, repair failures directly, evaluate implementation, create free-form prompts for core roles, or continue in degraded single-session fallback.

If required Codex project-scoped agents cannot be spawned, block the run before lifecycle role execution.

## Dispatch Semantics

Use:

```bash
bash .harness/scripts/dispatch-role.sh .harness/runs/<RUN_ID> <role>
```

`dispatch-role.sh` creates metadata only:

```txt
.harness/runs/<RUN_ID>/dispatch/<role>.dispatch.md
```

It does not spawn, execute, or emulate a subagent. Codex-native invocation of the named agent is the canonical execution path.

Before Planner dispatch, runtime capability may be manually asserted:

```bash
bash .harness/scripts/set-runtime-capability.sh .harness/runs/<RUN_ID> true
```

If subagent runtime is unavailable:

```bash
bash .harness/scripts/set-runtime-capability.sh .harness/runs/<RUN_ID> false "Subagent runtime unavailable"
```

Required blocked message:

```text
Subagent runtime unavailable.
Harness lifecycle requires Codex project-scoped subagents from `.codex/agents/`.
This run is blocked.
No lifecycle role may be executed in this session.
```

## Coordinator Write Scope

For coordinator/orchestrator sessions, run:

```bash
HARNESS_EXECUTOR_ROLE=coordinator \
HARNESS_RUN_DIR=".harness/runs/<RUN_ID>" \
bash .harness/scripts/validate-coordinator-write-scope.sh
```

Coordinator may write only narrow orchestration metadata allowed by the validator. `06-final-summary.md` must aggregate from `05-evaluator-report.md`, `04-implementation-report.md`, and `run.yaml`; it is not an independent role artifact.

## Role Boundaries

Planner writes `01-planner-brief.md` and `02-implementation-contract.md` in one planning invocation. Planner must not implement code, approve its own contract, or evaluate final output.

Contract Reviewer writes `03-contract-review.md`. Final line must be exactly `- Status: approved` or `- Status: rejected_requires_revision`.

Generator implements only after contract approval and writes `04-implementation-report.md`. Generator must stay within the approved contract and must not evaluate its own work.

Evaluator independently evaluates implementation and writes `05-evaluator-report.md`. Final line must be exactly `- Status: pass`, `- Status: fail`, or `- Status: blocked_insufficient_evidence`.

## Rework Routing

If Evaluator returns a non-passing result, Coordinator must not fix implementation directly. Create a bounded Generator rework packet from `.harness/templates/generator-rework-packet.template.md`, invoke `harness_generator`, then invoke `harness_evaluator` again.

If Generator cannot be spawned for required implementation or rework, stop with:

```text
BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
```

## Verification

Run verification through the appropriate role. Default guidance lives in `.harness/guides/TESTING_POLICY.md`.

Common commands, when relevant:

```bash
bash .harness/scripts/verify.sh
bash .harness/scripts/smoke.sh
```

## Legacy Artifacts Are Forbidden

Do not create or rely on:

```txt
HANDOFF.md
03-evaluator-contract-review.md
04-generator-worklog.md
06-fix-report.md
07-final-summary.md
```

Current canonical artifacts are `00` through `06-final-summary.md`.

## Priority

When instructions conflict, follow this order:

1. Current user request.
2. Root `AGENTS.md`.
3. Relevant `.harness/project/*`.
4. Relevant `.harness/codebase/*`.
5. `run.yaml` and current run artifacts.
6. Relevant `.codex/skills/harness-*/SKILL.md`.
7. Relevant `.harness/guides/*`.
8. Relevant `.harness/workflows/*`.
9. Relevant `.codex/agents/*.toml`.
10. Templates/scripts in `.harness/`.
11. Agent defaults or assumptions.
