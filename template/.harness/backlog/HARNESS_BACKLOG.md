# Harness Backlog

Harness Backlog là nơi ghi các đề xuất cải tiến harness sau khi quan sát agent làm việc.

File này thuộc target repository sau khi install. Target repository có quyền chỉnh sửa file này. Installer không overwrite `.harness/backlog/HARNESS_BACKLOG.md` nếu file đã tồn tại.

Một proposal tốt phải dẫn tới ít nhất một trong hai loại thay đổi:

1. Guide mới hoặc guide được sửa.
2. Sensor mới hoặc sensor được sửa.

Không ghi các nhận xét chung chung không thể kiểm chứng.

---

## Proposal Template

### ID

HB-0001

### Status

Proposed

### Problem

<Mô tả lỗi/lệch/lặp lại đã quan sát được>

### Evidence

- Run ID:
- File/report liên quan:
- Lỗi xảy ra ở đâu:

### Proposed Guide Change

<File guide nào cần thêm/sửa?>

```md
<Nội dung rule đề xuất>
```

### Proposed Sensor Change

<Test/linter/script/check nào cần thêm/sửa?>

```bash
<Lệnh hoặc mô tả sensor>
```

### Expected Benefit

<Việc này giúp agent tốt hơn như thế nào?>

### Acceptance Criteria

- [ ] Guide được cập nhật
- [ ] Sensor được thêm/sửa nếu cần
- [ ] Đã thử lại trên ít nhất 1 run
- [ ] Proposal được đóng hoặc chuyển thành follow-up

### Decision

- [ ] Accepted
- [ ] Rejected
- [ ] Needs More Evidence

### Notes

...
