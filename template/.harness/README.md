# Installed Harness

Thư mục này là Harness đã được cài vào target repository.

Nó thuộc quyền sở hữu của target repository sau khi install. Repo seed chỉ cung cấp bản khởi tạo; target repository có thể chỉnh `.harness/` để phù hợp với source code, stack, validation command, workflow local, và quyết định kỹ thuật của chính nó.

## Thành phần chính

```txt
.harness/
  INSTALLATION.md
  HARNESS_SKILLS.md
  guides/
  subagents/
  workflows/
  skills/
  templates/
  project/
  codebase/
  project-templates/
  scripts/
  backlog/
  runs/
```

- `HARNESS_SKILLS.md`: registry ngắn để agent chọn Harness workflow skill cần load.
- `guides/`: quy trình và policy cho agent.
- `subagents/`: fixed role templates for Planner, Contract Reviewer, Generator, and Evaluator.
- `workflows/`: default and Epic lifecycle specifications for strict template-based subagent orchestration.
- `skills/`: workflow skill file được load theo registry, không load toàn bộ theo mặc định.
- `templates/`: template artifact cho mỗi run.
- `project/`: project adapter của target repo. Installer chỉ tạo file thiếu, không overwrite file đã có.
- `codebase/`: source-navigation và change-impact cache của target repo. Installer chỉ tạo file thiếu, không overwrite file đã có. Layer này không thay thế hoặc duplicate `project/`.
- `project-templates/`: template trung lập dùng khi tạo project adapter mới.
- `scripts/`: helper scripts như `new-run.sh`, `inspect-project.sh`, `verify.sh`.
- `backlog/`: proposal cải tiến Harness local.
- `runs/`: execution namespace. Chứa normal runs và Epic containers. Epic là container điều phối task dài hơi, còn child runs trong Epic mới là đơn vị implementation có thể verify.

Trước khi tạo run, agent phải classify request bằng `guides/RUN_CLASSIFICATION.md`. Multi-phase, broad, long, MVP, full feature, core loop, hoặc task không verify gọn trong một run phải thành Epic.

Harness có ba lớp:

1. Artifact Protocol: run folders, templates, and evidence files.
2. Role Policy: Planner, Contract Reviewer, Generator, and Evaluator boundaries.
3. Lifecycle Orchestrator: `run.yaml`, `run-manifest.md`, state transitions, gates, `next-role.sh`, template-based subagent spawning, and `validate-run.sh`.

Harness core lifecycle execution requires real spawned subagents from predefined role templates.

Core lifecycle:

```txt
Planner -> Contract Reviewer -> Generator -> Evaluator
```

Coordinator phải gọi `.harness/scripts/dispatch-role.sh` cho role tiếp theo để tạo `.harness/runs/<RUN_ID>/dispatch/<role>.dispatch.md` bind fixed template trong `.harness/subagents/`. Script này chỉ tạo dispatch artifact; nó không spawn hoặc execute subagent. Runtime executor phải consume dispatch artifact rồi spawn subagent tương ứng. Coordinator không được tự làm lifecycle role, không được tạo prompt tự do cho core role, không được viết artifact thay role subagent, không được edit source/tests/config, và không được tự repair implementation failure.

New run bắt đầu với `runtime.subagent_runtime_available: unknown`. Trước Planner dispatch, mark runtime availability bằng `.harness/scripts/mark-subagent-runtime.sh`. Nếu runtime không thể spawn subagent, run phải block trước Planner execution. Không có degraded single-session fallback.

Nếu Evaluator trả kết quả không pass, Coordinator phải route qua bounded Generator rework packet rồi spawn Generator lại. Nếu không spawn được Generator, stop với `BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE`.

## Sau khi install

Ask your agent:

```txt
Read `.harness/HARNESS_SKILLS.md` and run the `project-sync` Harness workflow skill.
Then run `codebase-sync` if `.harness/codebase/*` is missing or stale.
```

Không cần cài native-agent skills.

Chạy verification mặc định:

```bash
bash .harness/scripts/verify.sh
```

Nếu app có runtime UI hoặc API:

```bash
APP_URL=http://localhost:5173 bash .harness/scripts/smoke.sh
```
