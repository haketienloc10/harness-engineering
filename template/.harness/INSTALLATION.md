# Harness Installation

File này là note template. Khi install vào target repository, installer sẽ tạo hoặc cập nhật `.harness/INSTALLATION.md` với thông tin cài đặt thực tế.

Ownership rules sau install:

- Target repository sở hữu `.harness/` của chính nó.
- `.harness/project/*` là project adapter local và không bị installer overwrite nếu đã tồn tại.
- `.harness/runs/RUN_INDEX.md`, normal runs, Epic containers, và child runs không bị reset.
- Legacy `.harness/epics/*`, nếu có từ Harness cũ, thuộc target repository và không bị installer xóa.
- `.harness/backlog/HARNESS_BACKLOG.md` không bị overwrite nếu target đã có.
- Kernel folders có thể được update khi chạy installer: `.harness/guides/`, `.harness/templates/`, `.harness/project-templates/`, `.harness/scripts/`.
- Seeded Harness workflow skill files được copy vào `.harness/skills/` mà không xóa skill file local khác.
