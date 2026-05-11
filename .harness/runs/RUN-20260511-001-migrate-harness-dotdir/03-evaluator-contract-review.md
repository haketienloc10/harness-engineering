# 03 Evaluator Contract Review

## Kết luận

Approved.

## Lý do

Contract có mục tiêu rõ ràng, phạm vi đo được, danh sách khu vực thay đổi cụ thể và kế hoạch kiểm chứng bằng grep plus script execution. Migration được giới hạn vào đổi layout workflow directory từ `.harness/` sang `.harness/` và không mở rộng sang thay đổi semantics Harness.

## Điều kiện khi implementation

- Không chỉnh logic Planner/Generator/Evaluator ngoài path migration.
- Bảo toàn nội dung `AGENTS.md` người dùng vừa thêm, chỉ cập nhật path cần thiết.
- Nếu phát hiện tham chiếu `.harness/` còn lại sau grep, phải phân loại là hợp lệ hay cần sửa trước khi pass.
