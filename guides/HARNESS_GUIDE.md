# Hướng dẫn Vận hành Quy trình Harness (Từ lý thuyết đến thực hành)

Hướng dẫn này được tổng hợp dựa trên luồng tài liệu xuất phát từ `AGENTS.md`,
giúp bạn hiểu và thực thi đúng vòng đời của một luồng công việc (task) trong dự
án.

---

## 1. Khởi động & Nắm bắt bối cảnh (AGENTS.md & README.md)

- **Nguyên tắc cốt lõi:** Luôn bắt đầu bằng việc đọc `AGENTS.md` (chỉ thị dự
  án). Tài liệu này sẽ dẫn bạn tới `docs/README.md` và `docs/HARNESS.md` để hiểu
  quy tắc cộng tác.
- **Công cụ chính:** Mọi hành động quản lý (intake, story, trace...) không được
  sửa file thủ công (trừ các file `.md` tài liệu/template) mà phải thông qua
  CLI: `scripts/bin/harness-cli` (lưu trữ tại SQLite db `harness.db`).

## 2. Tiếp nhận yêu cầu & Phân loại rủi ro (FEATURE_INTAKE.md)

Khi User đưa ra một yêu cầu mới, **không bắt tay vào code ngay**:

1. Phân loại yêu cầu (New spec, Bug fix, Maintenance...).
2. Đánh giá rủi ro theo checklist (chạm vào Auth, Data Model, API Public...):
   - **Tiny**: Lỗi nhỏ, typo, sửa doc. Sửa trực tiếp.
   - **Normal (1-3 flags rủi ro)**: Tính năng hẹp. Yêu cầu tạo/cập nhật
     `docs/templates/story.md`.
   - **High-Risk (4+ flags hoặc dính Hard gates như Auth/Data Loss)**: Phức tạp
     và rủi ro. Yêu cầu tạo hẳn một thư mục từ `docs/templates/high-risk-story/`
     bao gồm `execplan.md`, `overview.md`, `design.md`, và `validation.md`.
3. Khởi tạo tác vụ trên hệ thống bằng lệnh:
   ```bash
   scripts/bin/harness-cli intake --type <type> --summary <text> --lane <lane>
   ```

## 3. Khám phá & Lên thiết kế (ARCHITECTURE.md & HARNESS.md)

- **Kiến trúc:** Dựa theo `docs/ARCHITECTURE.md`, luôn tuân thủ nguyên tắc
  "Parse-First Boundary Rule" (kiểm tra và parse dữ liệu ở ranh giới) và
  "Dependency Rule" (lớp trong không phụ thuộc lớp ngoài).
- **Quyết định (Decision):** Đối với các tác vụ high-risk làm thay đổi kiến trúc
  hoặc bảo mật, hãy thảo luận với User. Sau khi chốt, tạo file dựa trên
  `docs/templates/decision.md` vào `docs/decisions/` và ghi nhận bằng lệnh:
  `scripts/bin/harness-cli decision add...`

## 4. Quản lý công việc và Kiểm thử (TEST_MATRIX.md)

- Thay vì mở rộng một file Spec khổng lồ, công việc được quản lý qua các
  **Stories**.
- Quản lý trạng thái story và mức độ bao phủ kiểm thử (unit, integration, e2e,
  platform) trên DB:
  - Tạo mới:
    `scripts/bin/harness-cli story add --id <id> --title <t> --lane <lane>`
  - Cập nhật:
    `scripts/bin/harness-cli story update --id <id> --unit 1 --integration 1...`
- **Xác thực:** Cấu hình lệnh kiểm tra tự động (VD: test command) và chạy
  `scripts/bin/harness-cli story verify <id>`.

## 5. Kết thúc & Lưu vết (TRACE_SPEC.md)

Sau khi hoàn thành code và update tài liệu sản phẩm:

- Bạn **bắt buộc** phải lưu lại quá trình làm việc bằng lệnh:
  `scripts/bin/harness-cli trace...`
- Mức độ chi tiết của Trace phụ thuộc vào Lane ban đầu:
  - **Minimal** (cho Tiny).
  - **Standard** (cho Normal).
  - **Detailed** (cho High-risk).
- Trace cần ghi rõ các actions bạn đã làm, file đã đọc, file đã sửa, quyết định
  đưa ra và quan trọng nhất là **friction**.

## 6. Ghi nhận khó khăn & Cải tiến quy trình (HARNESS_BACKLOG.md)

"Harness phát triển từ sự khó khăn" (Friction).

- Quá trình làm việc nếu gặp tài liệu lỗi thời, quy trình mâu thuẫn, kiểm thử
  thiếu hoặc thao tác lặp đi lặp lại bực mình, bạn phải ghi nó vào mục
  `--friction` của lệnh trace.
- Nếu vấn đề đó cần được sửa chữa hoặc cải tiến quy trình trong tương lai, hãy
  đưa nó vào backlog bằng lệnh:
  `scripts/bin/harness-cli backlog add --title "<tên>" --pain "<nỗi đau>"`
