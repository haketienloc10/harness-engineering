# Danh sách tài liệu quy trình Harness

Dựa trên quá trình bắt đầu từ file `AGENTS.md` và đọc theo chuỗi các file được
nhắc tới (không quét thư mục tự do), dưới đây là danh sách toàn bộ các file tài
liệu và cấu trúc liên quan đến quy trình (Harness process):

## 1. Điểm bắt đầu

- `AGENTS.md`: Chứa hướng dẫn tổng quan cho agent và danh sách các tài liệu cốt
  lõi đầu tiên.

## 2. Các file tài liệu cốt lõi

_(Được nhắc đến trực tiếp trong AGENTS.md và README.md)_

- `docs/README.md`: Bản đồ tài liệu tổng quan.
- `docs/HARNESS.md`: Mô hình hoạt động cốt lõi, cách con người và agent cộng
  tác, cùng vòng đời công việc.
- `docs/FEATURE_INTAKE.md`: Quy trình phân loại luồng công việc (tiny, normal,
  high-risk).
- `docs/ARCHITECTURE.md`: Các nguyên tắc về kiến trúc, khám phá trước khi định
  hình và ranh giới hệ thống.
- `docs/TEST_MATRIX.md`: Bản đồ ma trận kiểm thử (dùng làm template/quy tắc theo
  dõi).
- `docs/HARNESS_BACKLOG.md`: Quy trình ghi nhận thiếu sót/khó khăn (friction)
  trong quá trình làm việc để cải thiện Harness.
- `docs/GLOSSARY.md`: Từ điển thuật ngữ chung của toàn dự án.

## 3. Các file cấu hình/template mở rộng

_(Được nhắc đến bên trong các file cốt lõi ở trên)_

- `docs/TRACE_SPEC.md` (nhắc đến trong `HARNESS.md`): Đặc tả chi tiết về cách
  thức ghi log (trace) lại các hành động của agent (các cấp độ: Minimal,
  Standard, Detailed).
- `docs/templates/story.md` (nhắc đến trong `FEATURE_INTAKE.md`): Template chuẩn
  cho một story thông thường.
- `docs/templates/decision.md` (nhắc đến trong `HARNESS.md` và
  `FEATURE_INTAKE.md`): Template lưu lại các quyết định (Decisions/ADR) về thay
  đổi kiến trúc/hệ thống.
- `docs/templates/high-risk-story/execplan.md` (nhắc đến trong
  `FEATURE_INTAKE.md`): Kế hoạch thực thi cho các task rủi ro cao.
- `docs/templates/high-risk-story/overview.md` (nhắc đến trong
  `FEATURE_INTAKE.md`): Tổng quan cho task rủi ro cao.
- `docs/templates/high-risk-story/design.md` (nhắc đến trong
  `FEATURE_INTAKE.md`): Thiết kế cho task rủi ro cao.
- `docs/templates/high-risk-story/validation.md` (nhắc đến trong
  `FEATURE_INTAKE.md`): Quy trình xác thực cho task rủi ro cao.

## 4. File Database Schema & Tooling

_(Được nhắc đến trong HARNESS.md và TRACE_SPEC.md)_

- `scripts/schema/001-init.sql`: File chứa schema khởi tạo SQLite Database để
  lưu trữ dữ liệu vận hành thực tế (intake, story, decision, backlog, trace).
- `scripts/bin/harness-cli`: Công cụ dòng lệnh bằng Rust được nhắc đi nhắc lại
  trong hầu hết các file để tương tác với quy trình thay vì chỉnh sửa thủ công.

_(Lưu ý: Quá trình đọc cũng thấy một số tên file như `PHASE2.md`,
`src/auth/roles.ts`, `docs/decisions/0008-auth-boundary.md`,... nhưng sau khi
kiểm tra, đây chỉ là các chuỗi ví dụ/giả định được viết trong tài liệu để minh
hoạ cho các command chứ không phải là file chứa quy trình hệ thống hiện hữu)._
