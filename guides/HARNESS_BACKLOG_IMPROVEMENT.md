Dưới đây là hướng dẫn vận hành chi tiết đi sâu vào bước **Ghi nhận khó khăn &
Cải tiến quy trình**, được chắt lọc từ tài liệu `docs/HARNESS_BACKLOG.md` và
nguyên tắc _Growth Rule_ trong `docs/HARNESS.md`.

Quy trình của Harness không phải là tĩnh. Triết lý cốt lõi là **"The harness
grows from friction"** (Harness lớn lên từ nỗi đau). Bước này giúp chuyển hóa
các vướng mắc của hiện tại thành sự thông suốt cho tương lai.

---

# Hướng dẫn Vận hành Chi tiết: Ghi nhận Khó khăn & Cải tiến

## 1. Khi nào cần ghi nhận Cải tiến?

Trong lúc thi hành tác vụ (hoặc lúc đang viết lệnh Trace cuối bài), nếu
Agent/User phát hiện một trong những điều sau, thì **bắt buộc** phải ghi nhận
Backlog:

- Bị nhầm lẫn do tài liệu mâu thuẫn hoặc cũ kỹ.
- Phải tự suy diễn một bộ luật (rule) bị thiếu trong hệ thống.
- Nhận thấy thiếu một câu lệnh test (validation command) tự động.
- Phát hiện một mô hình lỗi lặp đi lặp lại.
- Phải làm thủ công một thao tác đáng ra có thể chuyển thành template hoặc
  checklist tự động.

_(Nếu lỗi đó nằm trong phạm vi có thể sửa ngay (ví dụ: chỉnh vài chữ trong
docs), Agent có thể tự sửa. Còn nếu đòi hỏi thêm template/công cụ/định hướng,
phải chuyển vào Backlog)._

## 2. Thao tác ghi nhận qua CLI

Dữ liệu khó khăn phải được ghi thẳng vào Database `harness.db` thông qua CLI,
không chỉ liệt kê thuần túy bằng văn bản Markdown.

**Câu lệnh khai báo cơ bản:**

```bash
scripts/bin/harness-cli backlog add --title "<Tên ngắn gọn của vấn đề>" --pain "<Mô tả chi tiết nỗi đau/sự bất tiện vừa gặp>"
```

**Ví dụ:**

```bash
scripts/bin/harness-cli backlog add --title "Thiếu quy tắc ủy quyền role" --pain "Tài liệu permission.md hiện không nói rõ cách xử lý Delegated Admin, khiến Agent phải tự đoán."
```

## 3. Vòng lặp Đo lường Kết quả (Backlog Outcome Loop)

Những cải tiến quan trọng thường đi kèm với kỳ vọng. Harness cung cấp cơ chế để
so sánh giữa "Kỳ vọng" và "Thực tế".

- **BƯỚC 1 - Ghi nhận dự đoán (Predicted):** Khi tạo một item, hãy dự báo tác
  động mong muốn của nó:
  `--predicted "Dự kiến sẽ giảm 50% thời gian tạo config nhờ có template mới."`
- **BƯỚC 2 - Ghi nhận thực tế (Outcome):** Khi task cải tiến này thực sự được
  triển khai và đóng lại, cập nhật bằng chứng thực tế:
  `--outcome "Template mới đã hoạt động, thời gian tạo config giảm xuống còn dưới 1 phút."`
- **BƯỚC 3 - Truy vấn kiểm tra (Query):**
  - Xem danh sách đang cần cải tiến:
    `scripts/bin/harness-cli query backlog --open`
  - Xem danh sách đã xử lý xong để đánh giá chênh lệch giữa dự đoán và thực tế:
    `scripts/bin/harness-cli query backlog --closed`

## 4. Phân loại Rủi ro cho Cải tiến

Sửa đổi quy trình cũng ẩn chứa rủi ro. Các mục Backlog cũng dùng chung từ vựng
phân loại rủi ro như lúc Intake: `tiny`, `normal`, và `high-risk`.

- Bạn có thể gán nhãn rủi ro bằng cờ: `--risk tiny`, `--risk normal`, hoặc
  `--risk high-risk`.
- **Tuyệt đối không dùng từ `low`**, hãy sử dụng `tiny` cho các rủi ro nhỏ gọn
  theo đúng chuẩn từ vựng hệ thống.

## 5. (Phụ lục) - Cấu trúc 1 Ticket Backlog hoàn chỉnh

Mặc dù CLI sẽ tự lo việc cấu trúc, nhưng khi được yêu cầu mô tả ra văn
bản/Markdown cho con người đọc, một báo cáo Backlog chuẩn (theo
`HARNESS_BACKLOG.md`) sẽ phải có đủ các trường:

- **Title:** Tên ngắn gọn (Short name).
- **Discovered While:** Phát hiện ra sự thiếu sót này khi đang làm Story/Task
  nào?
- **Current Pain:** Khó khăn cụ thể là gì (thủ công, lặp lại, mơ hồ, không an
  toàn)?
- **Suggested Improvement:** Giải pháp đề xuất thêm/đổi là gì?
- **Risk:** Mức độ rủi ro (tiny / normal / high-risk).
- **Status:** Trạng thái xử lý (proposed / accepted / implemented / rejected).
