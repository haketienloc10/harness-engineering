# Harness Installation

File này là note template. Khi install vào target repository, installer sẽ tạo hoặc cập nhật `.harness/INSTALLATION.md` với thông tin cài đặt thực tế.

Ownership rules sau install:

- Target repository sở hữu `.harness/` của chính nó.
- `.harness/project/*` là project adapter local và không bị installer overwrite nếu đã tồn tại.
- `.harness/codebase/*` là source-navigation và change-impact cache local và không bị installer overwrite nếu đã tồn tại.
- `.harness/runs/RUN_INDEX.md`, normal runs, Epic containers, và child runs không bị reset.
- Legacy `.harness/epics/*`, nếu có từ Harness cũ, thuộc target repository và không bị installer xóa.
- `.harness/backlog/HARNESS_BACKLOG.md` không bị overwrite nếu target đã có.
- Kernel folders có thể được update khi chạy installer: `.harness/guides/`, `.harness/workflows/`, `.harness/templates/`, `.harness/project-templates/`, `.harness/scripts/`.
- Lifecycle subagent definitions canonical nằm trong `.codex/agents/*.toml`; `.harness/subagents/` là deprecated và bị gỡ khi update.
- Harness workflow skills được cài vào `.codex/skills/harness-*/SKILL.md`; legacy `.harness/HARNESS_SKILLS.md` và seeded `.harness/skills/*.md` không còn là install source.
