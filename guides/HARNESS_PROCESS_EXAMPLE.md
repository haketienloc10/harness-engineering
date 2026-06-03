### GIAI ĐOẠN 0: Khởi động (Entrypoint)

**Tài liệu tham chiếu:** `AGENTS.md`

1. **Agent khởi động:** Đọc `AGENTS.md` đầu tiên.
2. **Nhận thức quy tắc cốt lõi:** - Biết rằng dự án này sử dụng "Harness".
* Công cụ chính để thao tác là `scripts/bin/harness-cli` (hoặc `.exe` trên Windows).
* Quy tắc định dạng: Tất cả file markdown phải hard-wrap ở 80 ký tự.


3. **Chuyển pha:** Agent chuyển sang đọc `CONTEXT_RULES.md` để biết mình cần đọc gì tiếp theo cho quá trình tiếp nhận yêu cầu (Intake Phase).

---

### GIAI ĐOẠN 1: Tiếp nhận yêu cầu (Intake Phase)

**Tài liệu tham chiếu:** `CONTEXT_RULES.md`, `FEATURE_INTAKE.md`

Agent nhận prompt: *"Hãy thêm tính năng Đăng nhập bằng Email và Password, trả về JWT token."*

1. **Phân loại Input (Input Type):** Đây là một "Change request" (Yêu cầu thay đổi) hoặc "New initiative" (Sáng kiến mới) vì nó thêm một tính năng hoàn toàn mới.
2. **Chạy Risk Checklist (Danh sách rủi ro):**
* Đụng chạm đến `login`, `password`, `JWT` -> Cờ rủi ro: **Auth** (Xác thực).
* Đụng chạm đến `schema` (lưu user) -> Cờ rủi ro: **Data model**.
* Cập nhật API -> Cờ rủi ro: **Public contracts**.


3. **Chọn Lane (Luồng làm việc):** Dựa theo `FEATURE_INTAKE.md`, vì chạm vào **Auth** (một "hard gate" - rào cản cứng), tác vụ này bắt buộc phải rơi vào luồng **High-Risk (Rủi ro cao)**.
4. **Ghi nhận vào hệ thống (Durable Layer):** Agent gọi CLI để lưu trạng thái intake.

```bash
scripts/bin/harness-cli intake \
  --type "Change request" \
  --summary "Add Email/Password JWT login" \
  --lane high-risk

```

*(Giả sử CLI trả về `intake_id: 42`)*

---

### GIAI ĐOẠN 2: Lập kế hoạch & Tạo Story (Planning Phase)

**Tài liệu tham chiếu:** `CONTEXT_RULES.md`, `ARCHITECTURE.md`, `HARNESS.md`

Vì đây là **High-Risk**, Agent phải đọc thêm các file liên quan đến kiến trúc và tạo một thư mục Story chi tiết, thay vì chỉ tạo một file markdown đơn giản.

1. **Tạo Story Packet:** Agent tạo thư mục `docs/stories/epics/E01-auth/US-001-jwt-login/` bao gồm các file `execplan.md`, `overview.md`, `design.md`, và `validation.md` từ template.
2. **Tuân thủ Architecture:**
* Đọc `ARCHITECTURE.md`: Áp dụng quy tắc "Parse-First Boundary" (phải validate email/password ở tầng Interface trước khi đưa vào Domain).
* Đảm bảo "Observability Contract": Thêm log chuẩn JSON khi đăng nhập thành công/thất bại (không log password).


3. **Ghi nhận Quyết định (Decision Record):** Thay đổi về Auth đòi hỏi một Durable Decision. Agent tạo file `docs/decisions/0001-jwt-auth-strategy.md` và ghi vào CLI:

```bash
scripts/bin/harness-cli decision add \
  --id 0001-jwt-auth-strategy \
  --title "Sử dụng JWT cho Authentication" \
  --doc docs/decisions/0001-jwt-auth-strategy.md \
  --notes "Quyết định sử dụng JWT token cho API authentication."

scripts/bin/harness-cli story add \
  --id US-001 \
  --title "Email/Password JWT Login" \
  --lane high-risk

```

---

### GIAI ĐOẠN 3: Triển khai Code (Implementation Phase)

**Tài liệu tham chiếu:** Các file code, `ARCHITECTURE.md`

1. **Viết Code:** Agent tiến hành viết mã nguồn tuân thủ "Dependency Rule" (Tầng Domain không phụ thuộc vào Framework/DB).
* *Interface:* Viết HTTP Controller, DTO parser (validate email format).
* *Application:* Viết Command xử lý Login.
* *Domain:* Tạo thực thể `User`, `UserId`.
* *Infrastructure:* Viết DB adapter để query user, bcrypt để hash password.


2. **Cập nhật Product Docs:** Tạo/Cập nhật `docs/product/auth.md` để phản ánh hợp đồng API mới.

---

### GIAI ĐOẠN 4: Xác thực & Bằng chứng (Validation Phase)

**Tài liệu tham chiếu:** `TEST_MATRIX.md`, `HARNESS.md`

Agent không được phép nói "hoàn thành" nếu chưa có bài test.

1. **Cập nhật Test Matrix:** Agent thêm dòng vào `docs/TEST_MATRIX.md` (hoặc thông qua CLI) đánh dấu tiến độ test.
2. **Chạy Test & Ghi nhận CLI:** Agent chạy Unit test (xử lý logic) và Integration test (kết nối DB thật).

```bash
# Cập nhật bằng chứng vào Database của Harness
scripts/bin/harness-cli story update \
  --id US-001 \
  --unit 1 \
  --integration 1 \
  --e2e 0 \
  --platform 0 \
  --status implemented

# Gắn lệnh verify cho Story
scripts/bin/harness-cli story update \
  --id US-001 \
  --verify "cargo test --package auth --all-features"

# Chạy lệnh kiểm tra cơ học (Verification Gate)
scripts/bin/harness-cli story verify US-001

```

---

### GIAI ĐOẠN 5: Ghi lại Dấu vết (Trace Phase)

**Tài liệu tham chiếu:** `TRACE_SPEC.md`, `GLOSSARY.md`

Vì đây là luồng **High-Risk**, Agent bắt buộc phải ghi lại một **Detailed Trace (Score: 3)**. Trace này phải bao gồm mọi quyết định, danh sách file, lượng token, thời gian và bất kỳ ma sát (friction) nào.

1. **Phát hiện Ma sát (Friction):** Trong lúc làm, Agent nhận ra tài liệu `ARCHITECTURE.md` chưa quy định rõ JWT nên có thời hạn (expiration) là bao lâu. Đây là một "Harness Friction".
2. **Lưu Trace qua CLI:**

```bash
scripts/bin/harness-cli trace \
  --summary "Đã hoàn thành High-risk JWT Login bao gồm Unit/Integration tests" \
  --intake 42 \
  --story US-001 \
  --agent gemini \
  --outcome completed \
  --duration 1200 \
  --tokens 35000 \
  --actions '["read ARCHITECTURE.md","drafted story US-001","created decision 0001","implemented auth layer","ran tests"]' \
  --read '["docs/ARCHITECTURE.md","docs/FEATURE_INTAKE.md","docs/templates/high-risk-story/"]' \
  --changed '["docs/stories/epics/E01-auth/US-001-jwt-login/","docs/decisions/0001-jwt-auth-strategy.md","src/auth/","docs/TEST_MATRIX.md"]' \
  --decisions '["Dùng bcrypt cho password hash","Tạo thư mục high-risk story riêng","Tách interface và domain layer cho auth"]' \
  --errors '["none"]' \
  --friction '["ARCHITECTURE.md thiếu quy chuẩn về thời hạn (expiration time) mặc định cho JWT. Đã đưa vào backlog."]' \
  --notes "Trace chi tiết cho tác vụ Auth. Yêu cầu test E2E sau khi có frontend."

```

*(Hệ thống Harness đánh giá Trace này đạt Score 3 - Hợp lệ cho High-risk).*

---

### GIAI ĐOẠN 6: Tiến hóa Hệ thống (Harness Growth)

**Tài liệu tham chiếu:** `HARNESS_BACKLOG.md`, `HARNESS.md`

Dựa trên nguyên tắc: *"The harness grows from friction"* (Harness phát triển từ những điểm nghẽn).
Agent đã tìm ra điểm nghẽn ở bước 5 (chưa có chuẩn thời gian JWT), do đó Agent phải đẩy nó vào Backlog để giải quyết sau.

```bash
scripts/bin/harness-cli backlog add \
  --title "Quy chuẩn thời hạn JWT" \
  --pain "ARCHITECTURE.md thiếu quy chuẩn về JWT expiration, dẫn đến agent phải tự đoán thời gian (ví dụ 1h hay 24h)." \
  --risk tiny \
  --predicted "Các tác vụ auth sau này sẽ có tham chiếu nhất quán, giảm 5% thời gian prompt."

```

---

### GIAI ĐOẠN 7: Hoàn thành (Done Definition)

**Tài liệu tham chiếu:** `HARNESS.md` (Mục Done Definition)

Agent tự kiểm tra Checklist lần cuối trước khi trả lời User:

* [x] Tính năng đã hoàn thành? (Code đã viết)
* [x] Tài liệu, Stories, Test Matrix đã cập nhật? (Đã cập nhật)
* [x] Validation commands đã chạy? (Đã chạy `story verify`)
* [x] Trace đã được lưu? (Đã lưu Detailed trace)
* [x] Ma sát đã được ghi vào Backlog? (Đã ghi backlog)

**Hành động cuối cùng:** Agent phản hồi lại người dùng (Human) bằng ngôn ngữ tự nhiên:

> *"Tôi đã hoàn thành việc triển khai tính năng Đăng nhập bằng Email/Password qua JWT theo luồng High-Risk. Story (US-001) và Quyết định Kiến trúc (0001) đã được tạo và ghi nhận vào cơ sở dữ liệu SQLite của Harness. Các bài Unit và Integration test đã pass. Tôi cũng đã phát hiện một thiếu sót trong `ARCHITECTURE.md` về thời hạn JWT và đã thêm nó vào Harness Backlog để xử lý trong tương lai. Bạn có thể kiểm tra ma trận test hiện tại bằng lệnh `scripts/bin/harness-cli query matrix`."*
