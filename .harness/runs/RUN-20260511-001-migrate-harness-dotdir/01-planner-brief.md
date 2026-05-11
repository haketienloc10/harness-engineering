# 01 Planner Brief

## Mục tiêu

Đổi layout Harness từ `.harness/` sang `.harness/` để thể hiện đây là workflow infrastructure/metadata, không phải application source tree.

## Phạm vi

- Đổi thư mục repository từ `.harness/` sang `.harness/`.
- Cập nhật mọi tham chiếu đường dẫn trong `AGENTS.md`, `README.md`, script, guide, template và bootstrap installer.
- Đảm bảo lệnh verify dùng đường dẫn mới.
- Duy trì run artifact cho chính migration này.

## Ngoài phạm vi

- Không thay đổi semantics Planner/Generator/Evaluator.
- Không thêm framework test mới.
- Không thay đổi logic workflow ngoài path migration cần thiết.

## Tiêu chí nghiệm thu

- Không còn tham chiếu bắt buộc tới `.harness/` cho workflow path hiện tại, trừ nội dung lịch sử trong run migration nếu cần.
- `.harness/scripts/verify.sh` chạy thành công.
- Installer cài workflow files vào `.harness/` trong repo đích.
- README và AGENTS.md hướng dẫn dùng `.harness/`.

## Khu vực có khả năng bị ảnh hưởng

- `AGENTS.md`
- `README.md`
- `scripts/install-harness.sh`
- `.harness/scripts/*`
- `.harness/guides/*`
- `.harness/templates/*`
- `.harness/runs/RUN_INDEX.md`

## Rủi ro và điểm chưa rõ

- Script bootstrap đang tìm `.harness/scripts/install.sh`, cần đổi sang `.harness/scripts/install.sh` sau khi rename.
- Các run lịch sử có thể chứa đường dẫn cũ; hiện chưa có run lịch sử ngoài migration này.
