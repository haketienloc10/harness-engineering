# AGENTS.md

Repository này đã cài **Harness** để điều phối AI-assisted development bằng artifact, role policy, và lifecycle state.

## Bootstrap Rules

- Reply theo ngôn ngữ người dùng; giữ code, command, path, API name, logs, và identifiers ở dạng gốc.
- Trước non-trivial work, đọc `.harness/HARNESS_SKILLS.md` và chỉ load skill liên quan.
- Trước lifecycle execution, đọc `.harness/guides/LIFECYCLE_ORCHESTRATION.md`.
- Nếu current agent runtime hỗ trợ independent subagent hoặc task execution, đọc `.harness/guides/SUBAGENT_EXECUTION.md` và dùng role-specific executor.
- Dùng `run.yaml` làm authoritative workflow state.
- Không sửa application code trước khi Contract Reviewer approve contract và `run.yaml` cho phép Generator.
- Evaluator phải độc lập với Generator và phải có evidence thật.
- Không kết thúc bằng generic “Suggested Next Steps” khi bước tiếp theo có thể được biểu diễn bằng subagent execution hoặc `HANDOFF.md`.

## Runtime-Agnostic Subagent Execution Requirement

Harness is agent-runtime agnostic.

When the current runtime provides independent subagent or task execution, Harness production roles MUST be executed by role-specific executors.

The current top-level agent is the Orchestrator. The Orchestrator may coordinate lifecycle state, but must not replace role executors.

Required role executors:
- Planner Agent -> dispatch to `planner`
- Contract Reviewer Agent -> dispatch to `contract-reviewer`
- Generator Agent -> dispatch to `generator`
- Evaluator Agent -> dispatch to `evaluator`

Do not simulate multiple production roles inside the same agent response when independent executors are available.

Do not create `HANDOFF.md` merely to cross a role boundary if an independent role executor can be started.

Create `HANDOFF.md` only when the runtime cannot start an independent role executor.

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
