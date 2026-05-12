# AGENTS.md

## Communication Defaults

- Reply in Vietnamese by default.
- Keep technical terms, code, commands, file paths, config keys, and error messages in their original form unless translation is explicitly requested.
- Prefer concise, operational Vietnamese.

## Repository Purpose

Repo này là **Harness Seed Architecture Repository**.

Repo này không phải application source code và không phải harness cứng cho một project cụ thể. Nguồn cài đặt canonical là:

```txt
template/.harness/
template/AGENTS.md
```

Sau khi install, target repository sở hữu `.harness/` của chính nó và có thể chỉnh tiếp độc lập.

## Repository Boundary

- `scripts/install-harness.sh` là bootstrap installer để tải repo seed và gọi installer canonical.
- `template/.harness/` là installed harness template.
- `template/AGENTS.md` là root agent instruction được cài vào target repo.
- Root repo không được có `.harness/` cạnh tranh với `template/.harness/` như install source.
- Template-based subagent orchestration policy lives in `template/AGENTS.md`, `template/.harness/subagents/`, and `template/.harness/workflows/`.

## Change Rules

Trước khi sửa:

- đọc file target trước;
- search usages nếu thay đổi behaviour của script hoặc template;
- nêu assumption nếu có nhiều cách hiểu;
- giữ thay đổi tối thiểu và đúng yêu cầu.

Khi chỉnh installer/template:

- không copy run history thật vào template;
- không làm installer overwrite `.harness/project/*` trong target repo;
- không reset `.harness/runs/RUN_INDEX.md` trong target repo;
- không overwrite `.harness/backlog/HARNESS_BACKLOG.md` nếu target đã có;
- chỉ replace kernel folders khi update: `.harness/guides/`, `.harness/subagents/`, `.harness/workflows/`, `.harness/templates/`, `.harness/project-templates/`, `.harness/scripts/`.
- khi thêm Harness kernel folder mới, installer và README phải liệt kê rõ folder đó.

## Verification

Với thay đổi installer/template, chạy tối thiểu:

```bash
bash -n scripts/install-harness.sh
bash -n template/.harness/scripts/*.sh
```

Nếu thay đổi installer, test bằng temp repo thật với install mới, idempotency/ownership-safe update, `preserve` mode, và `--dry-run`.

Nếu có `shellcheck`, chạy:

```bash
shellcheck scripts/install-harness.sh template/.harness/scripts/*.sh
```
