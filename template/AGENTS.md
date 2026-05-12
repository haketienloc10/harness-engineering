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
- Không kết thúc bằng generic “Suggested Next Steps” khi bước tiếp theo có thể được biểu diễn bằng executor dispatch.

## Role Execution

Harness uses orchestration, not handoff files, for role transitions.

The top-level agent is the Orchestrator.

The Orchestrator must spawn the required role-specific executor whenever the workflow enters a role-owned phase.

Required executors:

- `planner`
- `contract-reviewer`
- `generator`
- `evaluator`

The Orchestrator must not create `HANDOFF.md` to move between roles.

The Orchestrator must not simulate multiple production roles in one agent response.

If the current runtime cannot start an independent role executor, the run must be blocked unless `fallback_single_session_allowed: true` is explicitly set in `run.yaml`.

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
