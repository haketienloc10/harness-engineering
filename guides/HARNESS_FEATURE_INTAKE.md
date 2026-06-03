Dựa trên luồng quy trình tổng quát từ `AGENTS.md` và các tài liệu trong
`guides/*`, khâu **Tiếp nhận yêu cầu & Phân loại rủi ro** là chốt chặn (gate)
bắt buộc phải đi qua trước khi có bất kỳ thay đổi mã nguồn nào. Ở khâu này, AI
Agent (chứ không phải User) chịu trách nhiệm đánh giá và phân loại rủi ro.

Dưới đây là hướng dẫn vận hành chi tiết đi sâu vào quy trình **Feature Intake**
(`docs/FEATURE_INTAKE.md`):

---

# Hướng dẫn Vận hành Chi tiết: Tiếp nhận yêu cầu & Phân loại rủi ro (Feature Intake)

Mọi yêu cầu từ người dùng (từ dự án mới đến tính năng hoặc sửa lỗi) đều phải đi
qua luồng tiếp nhận (Intake Flow) với 5 bước cốt lõi: `User prompt` ➡️
`Phân loại Input Type` ➡️ `Định dạng thành Work Item` ➡️
`Tìm Docs/Stories liên quan` ➡️ `Chạy Risk Checklist` ➡️
`Chọn Lane (Tiny/Normal/High-risk)`.

## 1. Phân loại Yêu cầu đầu vào (Input Types)

Xác định loại yêu cầu để biết kết quả (artifact) cần tạo ra là gì trước khi chọn
luồng rủi ro:

- **New spec**: Chuyển spec do user cung cấp thành docs đạt chuẩn Harness. (Tạo
  ra: Product docs, candidate epics, decisions).
- **Spec slice**: Hiện thực hóa một phần hành vi từ spec đã duyệt. (Tạo ra:
  Story packet).
- **Change request**: Thay đổi, sửa lỗi, tinh chỉnh hành vi đã được duyệt. (Tạo
  ra: Story packet hoặc vá lỗi trực tiếp).
- **New initiative**: Thêm một tính năng/khu vực sản phẩm lớn cần nhiều stories.
  (Tạo ra: Initiative notes + story packets).
- **Maintenance request**: Thay đổi kỹ thuật, vận hành, hoặc thư viện phụ thuộc.
  (Tạo ra: Story packet, validation report, hoặc decision).
- **Harness improvement**: Cải thiện cách User và Agent cộng tác. (Sửa trực tiếp
  docs hoặc dùng `scripts/bin/harness-cli backlog add`).

**Lưu ý quan trọng**: Không tạo hay mở rộng một file Spec khổng lồ. Hãy sử dụng
Product docs, Stories, Decisions và Initiative notes làm tài liệu sống (living
surface).

## 2. Quét Checklist Rủi ro (Risk Checklist) & Cổng cứng (Hard Gates)

Đánh dấu 1 cờ (flag) cho mỗi yếu tố rủi ro bị ảnh hưởng:

1. **Auth**: Đăng nhập, đăng xuất, sessions, JWT, mật khẩu, refresh token.
2. **Authorization**: Quyền (roles, permissions), phạm vi công ty hoặc tenant.
3. **Data model**: Lược đồ database (schema), migrations, tính duy nhất, xóa,
   vòng đời dữ liệu.
4. **Audit/security**: Logs kiểm toán, quyền riêng tư, dữ liệu nhạy cảm, logs
   truy cập.
5. **External systems**: Email, thanh toán, dịch vụ cloud, SDKs bên thứ 3,
   queues, webhooks.
6. **Public contracts**: Cấu trúc API, response envelope, hành vi client nhìn
   thấy.
7. **Cross-platform**: Chia rẽ desktop/mobile/browser, hành vi native shell,
   deep links.
8. **Existing behavior**: Thay đổi hành vi đã có hoặc đã được test cover.
9. **Weak proof**: Thiếu test hoặc test không rõ ràng ở khu vực code bị ảnh
   hưởng.
10. **Multi-domain**: Thay đổi nhiều domain sản phẩm cùng lúc.

**Các Cổng cứng (Hard Gates) tự động đẩy task lên High-Risk:**

- Auth (Xác thực).
- Authorization (Phân quyền).
- Có rủi ro mất mát dữ liệu hoặc cần migration (Data loss/migration).
- Audit/security (Bảo mật/Kiểm toán).
- Hành vi của nhà cung cấp bên ngoài (External provider behavior).
- Gỡ bỏ hoặc làm yếu đi các yêu cầu xác thực dữ liệu (Validation requirements).

## 3. Phân luồng Mức độ Rủi ro (Lanes & Classification)

Dựa vào số cờ rủi ro (flags) hoặc cổng cứng (hard gates), task sẽ được xếp vào 1
trong 3 Lane sau:

### 3.1. Lane: Tiny (0-1 Flags)

- **Phạm vi áp dụng:** Lỗi nhỏ, sửa docs, sửa từ ngữ (copy), thay đổi siêu hẹp.
  Cài đặt project ban đầu (ví dụ: cài thư viện, setup server entrypoint, thêm
  health/smoke endpoint cục bộ) miễn là chưa tạo schema, CRUD, auth, hay tích
  hợp bên thứ ba.
- **Yêu cầu thực thi:**
  - Sửa code trực tiếp (Patch directly).
  - Cập nhật tài liệu nếu bị ảnh hưởng.
  - Chạy các công cụ quick checks.
  - Cập nhật lại harness (bằng `backlog add`) chỉ khi gặp vấn đề (friction).

### 3.2. Lane: Normal (1-3 Flags)

- **Phạm vi áp dụng:** Phạm vi ảnh hưởng có giới hạn (bounded blast radius), lớn
  bằng 1 Story. Nếu có 2-3 cờ rủi ro, cần chú trọng validation mạnh hơn.
- **Yêu cầu thực thi:**
  - **Bắt buộc** tạo hoặc cập nhật 1 file Story từ mẫu
    `docs/templates/story.md`.
  - Liên kết với các Product docs liên quan.
  - Thêm hoặc cập nhật kỳ vọng test (validation expectations).
  - Triển khai phần cắt dọc (vertical slice) nhỏ nhất nếu code đã có sẵn.
  - Ghi nhận trạng thái trên hệ thống CLI: `scripts/bin/harness-cli story add`
    và `scripts/bin/harness-cli story update`.

### 3.3. Lane: High-Risk (4+ Flags hoặc dính Hard Gates)

- **Phạm vi áp dụng:** Có dính cổng cứng (ví dụ: Auth) hoặc có tác động lan rộng
  đến bảo mật, dữ liệu, hợp đồng API, nền tảng. Có thể hạ cấp nếu User chủ động
  thu hẹp phạm vi rất chặt.
- **Yêu cầu thực thi:**
  - **Bắt buộc** tạo một thư mục riêng trong `docs/templates/high-risk-story/`.
  - Phải điền đầy đủ các file: `execplan.md`, `overview.md`, `design.md`, và
    `validation.md`.
  - Nếu định hướng không rõ ràng, **phải dừng lại và yêu cầu User xác nhận**
    trước khi code.
  - Nếu thay đổi kiến trúc, data ownership, API, bảo mật... phải ghi một Quyết
    định lưu trữ lâu dài (Durable decision) vào file `docs/decisions/NNNN-*.md`
    (lấy từ template `docs/templates/decision.md`), sau đó chạy lệnh ghi nhận
    `scripts/bin/harness-cli decision add`.

## 4. Kết quả đầu ra của khâu Intake (Output)

Cuối quá trình Intake, Agent không được im lặng lao vào code mà phải tổng kết
trực tiếp định hướng với định dạng chuẩn sau trước khi thi hành:

```text
Lane: <tiny | normal | high-risk>
Reason: <Liệt kê các flag/hard-gate bị vi phạm>
Docs: <Các file docs bị ảnh hưởng>
Story: <Đường dẫn tới file story/folder story, ví dụ: docs/stories/...>
Validation: <Mức độ test cần bổ sung: unit, integration, E2E>
```
