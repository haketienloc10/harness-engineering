# Agent-First Harness

Harness là lifecycle tooling cho agent; product files và user instructions vẫn
là nguồn sự thật cho sản phẩm.

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
