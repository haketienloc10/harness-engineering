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
  project-templates/
  scripts/
  backlog/
  epics/
  runs/
```

- `HARNESS_SKILLS.md`: registry ngắn để agent chọn Harness workflow skill cần load.
- `guides/`: quy trình và policy cho agent.
- `skills/`: workflow skill file được load theo registry, không load toàn bộ theo mặc định.
- `templates/`: template artifact cho mỗi run.
- `project/`: project adapter của target repo. Installer chỉ tạo file thiếu, không overwrite file đã có.
- `project-templates/`: template trung lập dùng khi tạo project adapter mới.
- `scripts/`: helper scripts như `new-run.sh`, `inspect-project.sh`, `verify.sh`.
- `backlog/`: proposal cải tiến Harness local.
- `epics/`: workstream/long-task coordination layer. Epic dùng cho task dài hơi, giữ roadmap, acceptance matrix, decision log, và run index.
- `runs/`: execution units. Run dùng cho đơn vị thực thi nhỏ có thể verify.

## Sau khi install

Ask your agent:

```txt
Read `.harness/HARNESS_SKILLS.md` and run the `project-sync` Harness workflow skill.
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
