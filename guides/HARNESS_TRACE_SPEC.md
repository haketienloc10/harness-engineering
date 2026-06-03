Dưới đây là hướng dẫn vận hành chi tiết đi sâu vào bước **Kết thúc & Lưu vết
(Trace Spec)**, được tổng hợp dựa trên tài liệu `docs/TRACE_SPEC.md` và
`docs/HARNESS.md`.

Đây là chốt chặn cuối cùng trước khi một Task được báo cáo là "Hoàn thành". Hệ
thống Harness không coi việc "viết code xong" là xong, mà phải lưu vết lại lịch
sử để các Agent sau hoặc con người có thể tái hiện bối cảnh.

---

# Hướng dẫn Vận hành Chi tiết: Kết thúc & Lưu vết

## 1. Bản chất của Trace (Lưu vết)

Lưu vết là thao tác ghi nhận lại toàn bộ "bộ nhớ làm việc" của một phiên
(session) vào cơ sở dữ liệu SQLite (`harness.db`). Nó cung cấp bằng chứng để
chấm điểm chất lượng Agent, tìm ra nguyên nhân lỗi (failure attribution) và là
cơ sở để cải tiến Harness sau này.

Lệnh thực thi luôn là: `scripts/bin/harness-cli trace ...`

## 2. Các Cấp độ Chất lượng (Quality Tiers & Lane Mapping)

Tùy vào luồng Rủi ro (Lane) ở khâu Intake, bạn phải tuân thủ cấp độ Trace tương
ứng. Cấp độ Trace càng cao thì dữ liệu đầu vào (các tham số của lệnh `trace`)
càng phải chi tiết.

### 2.1. Cấp độ Minimal (Dành cho luồng Tiny) - Điểm mục tiêu: 1

- **Chỉ áp dụng khi:** Chỉnh sửa vô cùng nhỏ, không đụng vào code (VD: sửa typo
  docs). Không áp dụng nếu có bất kỳ cản trở (friction) nào xảy ra.
- **Các trường bắt buộc tối thiểu:**
  - `--summary`: Tóm tắt tác vụ tối thiểu 10 ký tự.
  - `--outcome`: `completed`, `blocked`, `partial`, hoặc `failed`.

### 2.2. Cấp độ Standard (Dành cho luồng Normal) - Điểm mục tiêu: 2

- **Áp dụng cho:** Các task Normal (1 story), hoặc task Tiny nhưng có sửa đổi
  các file quan trọng.
- **Các trường bắt buộc phải có:**
  - Tất cả các trường của Minimal.
  - `--intake`: ID của intake đã tạo.
  - `--story`: Mã story.
  - `--agent`: Tên của công cụ/agent đang làm.
  - `--actions`: Danh sách các hành động đã làm (cách nhau bằng dấu phẩy).
  - `--read`: Danh sách các file đã đọc (cách nhau bằng dấu phẩy).
  - `--changed`: Danh sách các file đã sửa.
  - Phải có tối thiểu `--errors` (lỗi) HOẶC `--friction` (khó khăn).

### 2.3. Cấp độ Detailed (Dành cho luồng High-Risk) - Điểm mục tiêu: 3

- **Áp dụng cho:** Các task High-Risk liên quan đến bảo mật, kiến trúc, thay đổi
  lớn cần bằng chứng khắt khe.
- **Các trường bắt buộc phải có:**
  - Tất cả các trường của Standard.
  - `--decisions`: Quyết định đưa ra lúc làm (nhưng KHÔNG thay thế việc tạo file
    Decision trong thư mục `docs/decisions/`).
  - `--duration` và `--tokens`: Ước lượng thời gian và token (nếu có).
  - `--errors`: Phải khai báo, nếu không có lỗi nào thì **bắt buộc ghi là
    `"none"`**.
  - `--friction`: Tương tự errors, không có khó khăn thì ghi `"none"`.
  - `--notes`: Bổ sung thêm bối cảnh.

## 3. Giao thức Ghi nhận Khó khăn (Friction Capture Protocol)

"Harness lớn lên từ nỗi đau" (The harness grows from friction). Bất cứ cản trở
nào khiến bạn chậm lại đều phải được ghi nhận vào tham số `--friction`.

- **Trường hợp áp dụng:** Tài liệu cũ/mâu thuẫn, luật bị thiếu phải tự đoán, bài
  kiểm thử thiếu hoặc quá đắt để chạy, các thao tác tay bị lặp lại nhiều lần.
- **Quy tắc viết:** Ghi thẳng vào nỗi đau cụ thể thay vì nói chung chung.
  - ❌ _Sai:_ "docs confusing" (tài liệu khó hiểu).
  - ✅ _Đúng:_ "Existing permission docs did not define delegated admin; added
    backlog item for role glossary." (Tài liệu quyền hạn hiện tại không định
    nghĩa admin ủy quyền; đã thêm backlog để bổ sung từ điển role).
- **Kết nối với Backlog:** Nếu friction đó lớn và cần một Task mới trong tương
  lai để dọn dẹp, bạn **bắt buộc** phải chuyển nó thành Backlog bằng lệnh:
  `scripts/bin/harness-cli backlog add --title "<tên>" --pain "<nỗi đau>"`

## 4. Ví dụ một lệnh Trace tiêu chuẩn

**Ví dụ Trace cấp độ Standard (cho task Normal):**

```bash
scripts/bin/harness-cli trace \
  --summary "Added Phase 2 trace specification and Harness reference" \
  --intake 36 \
  --story US-004 \
  --agent antigravity \
  --outcome completed \
  --actions "read PHASE2.md,drafted TRACE_SPEC.md,updated HARNESS.md" \
  --read "PHASE2.md,docs/HARNESS.md" \
  --changed "docs/TRACE_SPEC.md,docs/HARNESS.md" \
  --friction "none"
```

_(Sau khi chạy lệnh `trace`, CLI sẽ trả về một con điểm (score). Hãy đối chiếu
điểm số này (1, 2, hoặc 3) với Lane của task. Nếu điểm thấp hơn kỳ vọng, hãy xem
xét lại các trường dữ liệu còn thiếu trước khi gửi response cuối cùng cho
User)._
