Dưới đây là hướng dẫn vận hành chi tiết đi sâu vào bước **Khám phá & Thiết kế
(Discovery & Design)**, được tổng hợp dựa trên các nguyên tắc cốt lõi từ
`docs/ARCHITECTURE.md` và luồng quy trình của `docs/HARNESS.md`.

Giai đoạn này là nhịp ngưng bắt buộc giữa khâu tiếp nhận yêu cầu và khâu viết
code, nhằm đảm bảo mọi dòng code đều có kiến trúc nhất quán và an toàn.

---

# Hướng dẫn Vận hành Chi tiết: Khám phá & Thiết kế

## 1. Khám phá Bối cảnh (Discovery Before Shape)

Trước khi định hình cấu trúc mã nguồn, Agent và User phải làm rõ các khía cạnh
sau:

- **Product surfaces:** Xác định các bề mặt tương tác (browser, mobile, CLI,
  API, background worker...).
- **Runtime stack:** Chốt ngôn ngữ, framework, database, queues, hosting. (Bất
  kỳ lựa chọn stack nào có tính ràng buộc lớn trong tương lai phải được ghi lại
  vào `docs/decisions/`).
- **Core domains:** Định nghĩa các khái niệm cốt lõi của sản phẩm với tên gọi và
  hợp đồng (contract) ổn định.
- **Boundary inputs:** Xác định tất cả các nguồn dữ liệu đầu vào (HTTP requests,
  webhooks, file tải lên, biến môi trường, payloads từ bên thứ ba...).
- **Validation ladder:** Vạch ra các bài kiểm tra nhỏ nhất có thể chứng minh
  stack hoạt động (unit, integration, e2e...).

## 2. Quy tắc Phụ thuộc và Phân lớp (Dependency Rule & Layering)

Kiến trúc hệ thống mặc định đi theo hướng **Clean Architecture** (Lớp trong
không được phụ thuộc vào lớp ngoài):

`Domain ⬅️ Application ⬅️ Infrastructure ⬅️ Interface ⬅️ App Surfaces`

- **Domain (Lõi):** Chứa logic nghiệp vụ thuần túy, KHÔNG được phụ thuộc vào
  framework, Database, hay UI.
- **Application:** Phụ thuộc vào Domain (ví dụ: các use-cases,
  commands/queries).
- **Infrastructure:** Phụ thuộc vào Domain & Application (ví dụ: Concrete
  Database clients, Logging, Third-party services).
- **Interface:** Phụ thuộc backend layers (ví dụ: Controllers, Routes, DTOs).
- **App Surfaces:** Giao diện hoặc client gọi API, không được truy cập trực tiếp
  vào lõi Domain.

_(Lưu ý: Không tự động tạo sẵn toàn bộ các thư mục này. Chỉ tạo khi Story hiện
tại thực sự cần đến nhánh kiến trúc đó)._

## 3. Quy tắc Ranh giới: "Parse-First Boundary Rule"

Đây là quy tắc bảo mật và kiến trúc quan trọng nhất khi xử lý dữ liệu: **Dữ liệu
lạ (unknown input) phải được phân tích (parse) và xác thực ngay tại ranh giới**
trước khi đi sâu vào hệ thống.

- **Các ranh giới thường gặp:** HTTP body/query, dữ liệu session, JWT claims,
  biến môi trường (ENV variables), webhooks từ bên thứ 3, và thậm chí là dữ liệu
  từ DB truy vấn qua client ngoài.
- **Luồng xử lý bắt buộc:** `Unknown Input` ➡️ `Parser/Validator` ➡️
  `Typed DTO/Command` ➡️ `Application Use Case` ➡️ `Domain Object`.
- Hệ thống bên trong (inner layers) chỉ làm việc với các kiểu dữ liệu nghiệp vụ
  có ý nghĩa (như `UserId`, `WorkspaceId`, `DateRange`) thay vì liên tục phải
  validate lại các chuỗi raw string.

## 4. Phân tách Lệnh và Truy vấn (Command/Query Boundary)

Nếu hệ thống có tính năng đọc/ghi, ngay cả khi Database đơn giản, mã nguồn phải
tách biệt:

- **Commands:** Chịu trách nhiệm thay đổi trạng thái và chứa các side-effects
  (như tạo audit log).
- **Queries:** Chỉ đọc dữ liệu và định dạng cho consumer, không làm thay đổi
  trạng thái.
- Các quy tắc kiểm tra quyền hoặc rule logic phải nằm ở Domain/Application,
  không nằm lỏng lẻo ngoài Controllers.

## 5. Hợp đồng Quan sát (Observability Contract)

Mọi thiết kế API và server phải thỏa mãn yêu cầu log chuẩn: Xuất 1 dòng log JSON
duy nhất (canonical log) cho mỗi request bao gồm ít nhất: `timestamp`, `level`,
`request_id`, `user_id` (nếu có), `action`, `duration_ms`, `status_code`,
`message`. _(Phân định rõ ràng: Audit Logs là lịch sử thay đổi để User xem, còn
Application Logs là để vận hành hệ thống. Không dùng cái này thay cái kia)._

## 6. Lưu vết Quyết định Thiết kế (Decision Records)

Đối với các tác vụ thuộc nhóm **High-Risk** làm thay đổi định hướng kiến trúc,
auth, mô hình dữ liệu lớn, hay public API, Agent **bắt buộc phải ghi nhận
Decision** (ADR - Architecture Decision Record).

**Quy trình 2 bước ghi nhận Decision:**

1. Tạo một file `.md` tại thư mục `docs/decisions/NNNN-<tên-ngắn>.md` (sử dụng
   template từ `docs/templates/decision.md`).
2. Ghi nhận chính thức vào hệ thống cơ sở dữ liệu của Harness bằng CLI:
   ```bash
   scripts/bin/harness-cli decision add \
     --id <Mã số ID, vd: 0008-auth-boundary> \
     --title "<Tiêu đề>" \
     --doc docs/decisions/<tên-file-ở-bước-1>.md \
     --notes "<Ghi chú tóm tắt lý do quyết định>"
   ```

_(Lưu ý: Việc liệt kê quyết định thiết kế bằng text vào mục `--decisions` khi
chạy lệnh trace log cuối task là tốt, nhưng KHÔNG THỂ thay thế cho quy trình tạo
hồ sơ Decision Records lâu dài này)._
