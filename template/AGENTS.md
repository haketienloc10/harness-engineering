# AGENTS.md

Repository này đã cài **Harness** để điều phối AI-assisted development bằng artifact, role policy, và lifecycle state.

## Bootstrap Rules

- Reply theo ngôn ngữ người dùng; giữ code, command, path, API name, logs, và identifiers ở dạng gốc.
- Trước non-trivial work, đọc `.harness/HARNESS_SKILLS.md` và chỉ load skill liên quan.
- Trước lifecycle execution, đọc `.harness/guides/LIFECYCLE_ORCHESTRATION.md`.
- Trước lifecycle execution, đọc `.harness/guides/SUBAGENT_EXECUTION.md` và `.harness/workflows/default-lifecycle.md`.
- Dùng `run.yaml` làm authoritative workflow state.
- Không sửa application code trước khi Contract Reviewer approve contract và `run.yaml` cho phép Generator.
- Evaluator phải độc lập với Generator và phải có evidence thật.
- Không kết thúc bằng generic “Suggested Next Steps” khi bước tiếp theo có thể được biểu diễn bằng executor dispatch.

## Template-Based Subagent Orchestration

Harness core lifecycle roles MUST be executed by separate spawned subagents.

The coordinator MUST instantiate each core role from its predefined template under `.harness/subagents/`.

Core lifecycle roles are:

1. Planner
2. Contract Reviewer
3. Generator
4. Evaluator

The coordinator MUST NOT create free-form prompts for these roles.

The coordinator MUST NOT execute these roles itself.

The coordinator is orchestration-only. It MUST NOT perform implementation, review, verification, debugging, or repair work directly.

The coordinator MAY pass task-specific inputs to the selected role template, including:

- original user request
- project context summary
- relevant files
- previous role artifacts
- acceptance criteria
- verification commands
- constraints

The coordinator MUST NOT modify:

- role responsibilities
- forbidden actions
- required artifacts
- output schema
- evidence requirements
- pass/fail criteria
- independence requirements

The coordinator MUST NOT modify application source files, tests, production configuration, generated production artifacts, or project implementation files. Any source/test/config change MUST be performed by the Generator role.

For coordinator/orchestrator sessions, run the write-scope validator before accepting changed files:

```bash
HARNESS_EXECUTOR_ROLE=coordinator \
HARNESS_RUN_DIR=".harness/runs/<RUN_ID>" \
bash .harness/scripts/validate-coordinator-write-scope.sh
```

The validator enforces the narrow orchestration metadata allowlist documented in `.harness/guides/SUBAGENT_EXECUTION.md#coordinator-write-scope-validator`. If it fails, stop with `BLOCKED_COORDINATOR_WRITE_SCOPE_VIOLATION` and route source/test/config work to Generator.

If subagent spawning is unavailable, the run MUST be blocked.

There is no degraded single-session fallback.

## No Subagent Runtime, No Harness Run

Harness lifecycle execution requires real subagent spawning.

If the current runtime cannot spawn subagents, the coordinator MUST block the run before Planner execution.

The coordinator MUST NOT emulate subagents by writing multiple role artifacts in one session.

The coordinator MUST NOT continue in degraded mode.

A blocked run is valid and preferable to a fake multi-role run.

Required blocking message:

```text
Subagent runtime unavailable.
Harness lifecycle requires template-based subagent orchestration.
This run is blocked.
No lifecycle role may be executed in this session.
```

## Role Independence Audit

A Harness run is invalid if any of the following is true:

- Planner, Contract Reviewer, Generator, and Evaluator were performed by the same session.
- The coordinator wrote lifecycle artifacts on behalf of role subagents.
- A core role was executed from a free-form prompt instead of a role template.
- The run continued after detecting unavailable subagent runtime.
- Evaluator approved without independent evidence.

## Role Execution

Harness uses template-based subagent orchestration, not handoff files, for role transitions.

The top-level agent is the coordinator/orchestrator.

The coordinator must spawn the required role-specific subagent whenever the workflow enters a role-owned phase.

Required role templates:

- `.harness/subagents/planner.md`
- `.harness/subagents/contract-reviewer.md`
- `.harness/subagents/generator.md`
- `.harness/subagents/evaluator.md`

The coordinator must not create `HANDOFF.md` to move between roles.

The coordinator must not emulate multiple production roles in one agent response.

## Evaluator Failure Routing

When Evaluator returns `FAIL`, `REJECTED`, `NEEDS_FIX`, `blocked_insufficient_evidence`, or any equivalent non-passing result, the coordinator MUST NOT fix the implementation directly.

The coordinator may read only the evaluator decision summary, create a bounded Generator rework packet from `.harness/templates/generator-rework-packet.template.md`, spawn Generator, wait for Generator output, and then spawn Evaluator again.

If the runtime cannot spawn Generator for required implementation or rework, stop with:

```text
BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
```

## Execution Namespace

```txt
.harness/runs/RUN-*                 # normal runs
.harness/runs/EPIC-*                # Epic containers
.harness/runs/EPIC-*/runs/RUN-*     # child runs
```

Không tạo primary Epic data trong `.harness/epics/*`; nếu tồn tại thì chỉ coi là legacy/read-only.

## Priority

1. Current user request.
2. Root `AGENTS.md`.
3. `.harness/project/*` và `.harness/codebase/*` liên quan.
4. `run.yaml` và artifact trong current run.
5. Relevant `.harness/guides/*`.
6. Templates/scripts trong `.harness/`.

Harness là seed workflow layer, không phải application source tree. Khi làm task, inspect source code và runtime thực tế của target repository bên ngoài `.harness/`.
