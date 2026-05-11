# Parallel Work

## Một task, một run

Nếu user đưa nhiều task không liên quan, tạo một run cho mỗi task.

## Conflict check

Trước implementation, dùng:

```bash
bash .harness/scripts/check-conflicts.sh RUN-YYYYMMDD-NNN-task-slug
```

Nếu các run có thể sửa cùng file hoặc cùng module, ghi conflict vào contract và chọn một hướng:

- sequence work;
- dùng branch riêng;
- dùng worktree riêng;
- block cho đến khi rõ ownership.

## Không proceed silently

Nếu conflict có thể gây overwrite hoặc làm mất thay đổi của user/run khác, dừng và surface tradeoff.
