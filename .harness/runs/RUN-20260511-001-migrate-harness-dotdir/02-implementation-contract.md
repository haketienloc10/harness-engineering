# 02 Implementation Contract

## Mục tiêu

Migration toàn bộ Harness workflow directory từ `.harness/` sang `.harness/` và cập nhật mọi đường dẫn vận hành tương ứng.

## Thay đổi dự kiến

- Rename directory `.harness/` thành `.harness/`.
- Cập nhật references trong root instructions và docs từ `.harness/...` sang `.harness/...`.
- Cập nhật scripts để dùng `.harness` làm default workflow directory.
- Cập nhật bootstrap installer để tìm và chạy `.harness/scripts/install.sh`.
- Cập nhật install script để copy `.harness/guides`, `.harness/templates`, `.harness/scripts` vào `.harness/` ở target repo.

## Ngoài phạm vi

- Không hỗ trợ song song cả `.harness/` và `.harness/` trừ khi cần cho bootstrap từ archive hiện tại.
- Không đổi tên repository, package, hoặc remote URL `harness-engineering`.
- Không tạo application-level tests.

## File hoặc khu vực dự kiến thay đổi

- `AGENTS.md`
- `README.md`
- `scripts/install-harness.sh`
- `.harness/guides/*.md`
- `.harness/templates/*.md`
- `.harness/templates/run.yaml.template`
- `.harness/scripts/*.sh`
- `.harness/runs/RUN_INDEX.md`
- `.harness/runs/RUN-20260511-001-migrate-harness-dotdir/*`

## Rủi ro conflict

- Người dùng đã sửa `AGENTS.md`; chỉ thay đổi các path liên quan migration và giữ nguyên nội dung boundary đã thêm.
- Rename thư mục có blast radius lớn, cần grep sau migration để phát hiện path cũ còn sót.

## Behaviour Contract

- Agent đọc `AGENTS.md` sẽ thấy `.harness/` là workflow infrastructure, không phải application source tree.
- Lệnh `bash .harness/scripts/verify.sh` hoạt động trong repo sau migration.
- Lệnh installer cài workflow vào `.harness/` ở project đích.
- Lệnh tạo run mới tạo trong `.harness/runs/`.
- Lệnh list/check-conflicts đọc `.harness/runs/`.

## Kế hoạch kiểm chứng

- Chạy `grep -R ".harness/\|/harness" -n AGENTS.md README.md scripts .harness` và xác nhận chỉ còn reference hợp lệ tới repo name hoặc nội dung lịch sử nếu có.
- Chạy `bash .harness/scripts/verify.sh`.
- Chạy `bash .harness/scripts/list-runs.sh`.

## Giả định

- Không cần tương thích ngược với thư mục `.harness/` trong repo hiện tại sau migration.
- Repo đích mới sẽ dùng `.harness/`.
