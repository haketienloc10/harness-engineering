Dưới đây là hướng dẫn vận hành chi tiết đi sâu vào bước **Quản lý công việc và
Kiểm thử (Quản lý Story & Test Matrix)**, được đúc kết từ `docs/TEST_MATRIX.md`
và `docs/HARNESS.md`.

Bước này đảm bảo mọi đoạn code sinh ra đều được đóng gói thành các đơn vị công
việc rõ ràng (Story) và được chứng minh bằng các bài kiểm thử thực tế.

---

# Hướng dẫn Vận hành Chi tiết: Quản lý công việc & Kiểm thử

## 1. Nguyên tắc Quản lý theo Story

Trong hệ thống Harness, thay vì duy trì một file Spec khổng lồ, mọi tác vụ đều
được chia nhỏ thành các **Story Packets** (lưu tại `docs/stories/`).

### Các trạng thái của một Story:

Mỗi Story luôn mang 1 trong 5 trạng thái vòng đời sau:

- `planned`: Yêu cầu đã được duyệt nhưng chưa code.
- `in_progress`: Đang trong quá trình triển khai code.
- `implemented`: Đã hoàn thành code VÀ đã có bằng chứng (evidence/tests). (Tuyệt
  đối không set trạng thái này nếu chưa có test/bằng chứng).
- `changed`: Hợp đồng/Yêu cầu thay đổi sau khi đã code xong.
- `retired`: Tính năng không còn thuộc hợp đồng sản phẩm nữa.

### Thao tác vận hành CLI:

Mọi thay đổi trạng thái phải được cập nhật vào Database thông qua CLI:

- Thêm Story mới:
  `scripts/bin/harness-cli story add --id <id> --title <t> --lane <lane>`
- Cập nhật trạng thái:
  `scripts/bin/harness-cli story update --id <id> --status <status>`

## 2. Ma trận Kiểm thử (Test Matrix & Quy tắc Bằng chứng)

Bản chất của Test Matrix là **ánh xạ giữa hành vi sản phẩm và bằng chứng**. Có 4
cấp độ kiểm thử (Evidence Rules) bạn cần phải khai báo cho một Story:

- **Unit:** Dùng để kiểm chứng logic nội tại thuần túy của lớp Domain và
  Application (không dính tới DB hay I/O).
- **Integration:** Dùng để kiểm chứng backend (bảo mật, tính toàn vẹn dữ liệu,
  giao tiếp với provider bên thứ 3, jobs, và database).
- **E2E (End-to-End):** Kiểm chứng luồng hành vi hiển thị trực tiếp cho người
  dùng (thường qua trình duyệt).
- **Platform:** Cấp độ kiểm chứng cao nhất dành cho hành vi của hệ điều hành
  (shell), triển khai (deployment), app mobile/desktop mà các tầng dưới không
  giả lập được.

_(Lưu ý: Không phải Story nào cũng bắt buộc có đủ 4 loại test. Nhưng nếu bỏ qua
loại nào, bạn phải giải thích lý do bên trong file Markdown của Story đó)._

### Thao tác vận hành CLI:

Để khai báo mức độ bao phủ test cho một Story, bạn sử dụng các số `1` (có) và
`0` (không). CLI không nhận các chữ "yes/no":

```bash
scripts/bin/harness-cli story update --id <id> --unit 1 --integration 1 --e2e 0 --platform 0
```

Để xem tổng quan Ma trận Kiểm thử:

- Dạng dễ đọc (yes/no): `scripts/bin/harness-cli query matrix`
- Dạng số để tiện update: `scripts/bin/harness-cli query matrix --numeric`

## 3. Xác thực tự động (Story Verification)

Một tính năng mạnh mẽ của Harness là gắn thẳng câu lệnh kiểm thử (test command)
cơ học vào một Story, ví dụ: `npm run test` hoặc `cargo test`.

- **Gắn lệnh xác thực:**
  `scripts/bin/harness-cli story update --id <id> --verify "<câu lệnh kiểm thử>"`
- **Chạy lệnh xác thực:** `scripts/bin/harness-cli story verify <id>`

Khi chạy lệnh `verify`, Harness CLI sẽ thực thi lệnh test từ gốc thư mục
(repository root). Nếu thành công (exit code 0) hoặc thất bại (exit code 1), CLI
sẽ tự động ghi log vào các trường `last_verified_at` và `last_verified_result`
trong Database.

**Lưu ý quan trọng trước khi đóng task:** Nếu một Story được cấu hình lệnh
`verify`, nhưng lệnh đó chưa bao giờ trả về thành công (pass), thì ở bước cuối
cùng (khi chạy lệnh lưu vết `trace`), hệ thống sẽ phát ra cảnh báo. Do đó, hãy
luôn đảm bảo lệnh `story verify <id>` đã chạy pass trước khi kết thúc công việc!
