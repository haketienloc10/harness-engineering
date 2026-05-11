# 04 Generator Worklog

## Thay đổi đã thực hiện

- Đổi thư mục `harness/` thành `.harness/`.
- Cập nhật `AGENTS.md` để trỏ guide, templates, runs, scripts và backlog sang `.harness/`.
- Cập nhật `README.md` để hướng dẫn chạy verify/install theo `.harness/`.
- Cập nhật bootstrap `scripts/install-harness.sh` để tìm `.harness/scripts/install.sh` trong archive.
- Cập nhật `.harness/scripts/install.sh` để cài workflow files vào `.harness/` trong target repo.
- Cập nhật `.harness/scripts/new-run.sh`, `.harness/scripts/list-runs.sh`, `.harness/scripts/check-conflicts.sh` để dùng `.harness/runs/`.
- Cập nhật guide và template path references sang `.harness/`.

## Ghi chú

Một replacement tự động ban đầu làm sai biến shell `target_harness/guides` thành `target_.harness/guides` và làm artifact migration có `..harness/`; các lỗi này đã được phát hiện bằng grep/read và sửa lại trước verification.
