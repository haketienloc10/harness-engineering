# harness-engineering

Template để cài Harness vào repo khác, giúp AI-assisted development có plan, contract, worklog và evaluator report có thể kiểm chứng.

## Dành cho agent

Nếu người dùng chỉ gửi link repo này, hãy đọc README này rồi cài Harness vào repo đích. Cách nhanh nhất là chạy lệnh `curl | bash` từ thư mục repo đích:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/scripts/install-harness.sh?$(date +%s)" \
  | bash -s -- --target "$(pwd)" --agents-mode merge
```

Nếu repo đích đã có `AGENTS.md` và cần giữ nguyên để người dùng tự merge:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/scripts/install-harness.sh?$(date +%s)" \
  | bash -s -- --target "$(pwd)" --agents-mode preserve
```

Sau khi cài, chạy:

```bash
bash .harness/scripts/verify.sh
```

## Cài từ checkout

```bash
bash .harness/scripts/install.sh --target /path/to/your-repo --agents-mode merge
```

Không hỏi lại:

```bash
bash .harness/scripts/install.sh --target /path/to/your-repo --agents-mode preserve --yes
```

## Mode cho AGENTS.md

Installer không ghi đè `AGENTS.md` im lặng. Chọn một mode:

- `merge`: backup file cũ, rồi tạo `AGENTS.md` hợp nhất.
- `preserve`: giữ nguyên `AGENTS.md`, ghi Harness vào `AGENTS.harness.md`.
- `replace`: backup `AGENTS.md` hiện có, rồi thay bằng instruction Harness.
- `backup`: backup file cũ, rồi cài instruction Harness.

Nếu dùng `--yes` mà không truyền `--agents-mode`, khi target đã có `AGENTS.md`, script chọn `preserve`.

## Nội dung được cài

- `AGENTS.md` hoặc `AGENTS.harness.md`, tùy mode.
- `.harness/guides/`
- `.harness/templates/`
- `.harness/scripts/`
- `.harness/backlog/HARNESS_BACKLOG.md` nếu target chưa có file này.
- `.harness/runs/RUN_INDEX.md` sạch cho repo đích.

Installer không copy các run lịch sử dạng `.harness/runs/RUN-*` từ template repo sang repo đích.
