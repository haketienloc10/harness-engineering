# Harness Installation

File này là note template. Khi install vào target repository, installer sẽ tạo hoặc cập nhật `.harness/INSTALLATION.md` với thông tin cài đặt thực tế.

Ownership rules sau install:

- Target repository sở hữu `.harness/` của chính nó.
- `.harness/project/*` là project adapter local và không bị installer overwrite nếu đã tồn tại.
- `.harness/runs/RUN_INDEX.md` và run history local không bị reset.
- `.harness/epics/*` belongs to target repository.
- Installer does not reset `EPIC_INDEX.md` and does not copy epic history from seed.
- `.harness/backlog/HARNESS_BACKLOG.md` không bị overwrite nếu target đã có.
- Kernel folders có thể được update khi chạy installer: `.harness/guides/`, `.harness/templates/`, `.harness/project-templates/`, `.harness/scripts/`.
- Seeded Harness workflow skill files được copy vào `.harness/skills/` mà không xóa skill file local khác.
