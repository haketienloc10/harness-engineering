# harness-engineering

`harness-engineering` là **Harness Seed Architecture Repository**.

Repo này không phải application source code và không phải harness cứng cho một project cụ thể. Nguồn cài đặt canonical là:

```txt
template/.harness/
template/.codex/
template/AGENTS.md
```

Sau khi install, target repository sở hữu `.harness/` và `.codex/` Harness files của chính nó.

## Kiến Trúc

Harness seed cài bốn lớp:

- `.codex/agents/`: Codex project-scoped lifecycle agents.
- `.codex/skills/harness-*/SKILL.md`: workflow skills để load theo nhu cầu.
- `.harness/`: artifact protocol, guides, workflows, templates, scripts, runs, backlog, project/codebase context.
- `AGENTS.md`: bootstrap instruction cho target repo.

Core lifecycle agent names:

```txt
harness_planner
harness_contract_reviewer
harness_generator
harness_evaluator
```

Role responsibility canonical nằm trong `.codex/agents/*.toml`. Workflow/task guidance canonical nằm trong `.codex/skills/harness-*/SKILL.md`. `.harness/workflows/*` chỉ mô tả state transition và required artifacts.

## Cài Nhanh

Chạy từ target repository:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/scripts/install-harness.sh?$(date +%s)" \
  | bash -s -- --target "$(pwd)" --agents-mode merge
```

Giữ nguyên `AGENTS.md` để tự merge:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/scripts/install-harness.sh?$(date +%s)" \
  | bash -s -- --target "$(pwd)" --agents-mode preserve
```

Cài từ checkout:

```bash
bash template/.harness/scripts/install.sh --target /path/to/your-repo --agents-mode merge
```

Preview:

```bash
bash template/.harness/scripts/install.sh --target /path/to/your-repo --agents-mode merge --dry-run
```

## AGENTS.md Modes

- `ask`: hỏi tương tác nếu target đã có `AGENTS.md`.
- `merge`: backup file cũ rồi merge Harness instructions.
- `preserve`: giữ `AGENTS.md`, ghi Harness vào `AGENTS.harness.md`.
- `replace`: backup rồi thay bằng Harness `AGENTS.md`.
- `backup`: alias có chủ ý giống `replace`.

Nếu dùng `--yes` mà không truyền `--agents-mode`, installer chọn `merge`.

## Installed Layout

```txt
target-repo/
  AGENTS.md hoặc AGENTS.harness.md
  .codex/
    config.toml
    agents/
      harness-planner.toml
      harness-contract-reviewer.toml
      harness-generator.toml
      harness-evaluator.toml
    skills/
      harness-run-classification/SKILL.md
      harness-epic/SKILL.md
      harness-project-sync/SKILL.md
      harness-codebase-sync/SKILL.md
      harness-lifecycle/SKILL.md
  .harness/
    README.md
    INSTALLATION.md
    guides/
    workflows/
    templates/
    project-templates/
    project/
    codebase/
    scripts/
    backlog/
    runs/
```

## Ownership Rules

- `.harness/project/*`, `.harness/codebase/*`, `.harness/runs/*`, and `.harness/backlog/HARNESS_BACKLOG.md` are target-owned.
- Installer creates missing project/codebase files only and preserves run history/backlog.
- `.codex/config.toml` is created if missing; if present, only missing Harness `[agents]` defaults are merged after backup.
- `.codex/agents/harness-*.toml` and `.codex/skills/harness-*` are Harness-owned and same-name paths are backed up before overwrite.
- Kernel folders replaced on explicit update: `.harness/guides/`, `.harness/workflows/`, `.harness/templates/`, `.harness/project-templates/`, `.harness/scripts/`.
- Deprecated `.harness/subagents/`, `.harness/HARNESS_SKILLS.md`, and seeded `.harness/skills/*.md` are no longer install sources.

## Sau Khi Cài

Ask your agent:

```txt
Use .codex/skills/harness-project-sync/SKILL.md to refresh project context.
Then use .codex/skills/harness-codebase-sync/SKILL.md if source-navigation or change-impact docs are missing or stale.
```

Verification tối thiểu cho seed repo:

```bash
bash -n scripts/install-harness.sh
bash -n template/.harness/scripts/*.sh
```
