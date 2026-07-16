# Agent-First Harness

Harness là lifecycle tooling cho agent; product files và user instructions vẫn
là nguồn sự thật cho sản phẩm.

## Chất lượng câu trả lời

Tránh các câu trả lời trừu tượng hoặc chung chung.

Khi giải thích quyết định, kế hoạch, rủi ro, lỗi, kiến trúc hoặc sự đánh đổi,
hãy dùng ví dụ cụ thể và trình bày theo từng bước quan hệ nguyên nhân-kết quả.

Khi phù hợp, ưu tiên cấu trúc sau:

1. Điều gì xảy ra
2. Vì sao điều đó xảy ra
3. Ví dụ cụ thể
4. Tác động kéo theo
5. Hành động được khuyến nghị

1. Trước khi sửa, chạy `_harness/bin/harness-cli task start` với input, flags,
   owner và session phù hợp.
2. Đọc context và hoàn tất gates mà command trả về; chỉ đọc thêm product docs,
   stories hoặc decisions khi context yêu cầu.
3. Dùng `proof run` cho verification và `task finish` trước final response.
4. Xin human approval cho direction high-risk, credentials, chi phí, destructive
   action, hoặc gate mà policy yêu cầu.
5. Nếu CLI không có, không tạo operational DB thủ công: dùng Markdown artifact,
   chạy validation khả dụng và ghi rõ `harness-cli` missing như friction.

Không bỏ qua validation hoặc completion gates. Chỉ dùng command-first path để
thay đổi lifecycle state.
