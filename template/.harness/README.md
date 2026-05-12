# Installed Harness

Thư mục này là Harness đã được cài vào target repository.

Nó thuộc quyền sở hữu của target repository sau khi install. Repo seed chỉ cung cấp bản khởi tạo; target repository có thể chỉnh `.harness/` để phù hợp với source code, stack, validation command, workflow local, và quyết định kỹ thuật của chính nó.

## Thành phần chính

```txt
.harness/
  INSTALLATION.md
  HARNESS_SKILLS.md
  guides/
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
3. Lifecycle Orchestrator: `run.yaml`, state transitions, gates, `next-role.sh`, `HANDOFF.md`, and `validate-run.sh`.

Production workflow dùng các session/agent riêng cho Planner, Contract Reviewer, Generator, và Evaluator. Subagents là executor cho từng role, không thay thế lifecycle state machine. Nếu subagents hoặc session độc lập không có sẵn, tạo `HANDOFF.md` và dừng ở role boundary thay vì ghi “Suggested Next Steps” chung chung.

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
