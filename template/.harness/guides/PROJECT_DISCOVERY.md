# Project Discovery

## Mục đích

Project discovery tạo project adapter ban đầu cho target repository. Output là observed facts tại thời điểm chạy script, không phải absolute truth.

## Command

```bash
bash .harness/scripts/inspect-project.sh
```

Script update generated sections trong:

```txt
.harness/project/PROJECT_MAP.md
.harness/project/STACK_PROFILE.md
.harness/project/VALIDATION_PROFILE.md
```

Manual notes bên ngoài marker phải được giữ lại:

```txt
<!-- HARNESS:GENERATED:START -->
...
<!-- HARNESS:GENERATED:END -->
```

## Cách dùng evidence

- Treat discovery output as clues.
- Verify bằng source code, config, tests, docs, và runtime behaviour khi cần.
- Nếu discovery sai hoặc thiếu, chỉnh manual notes trong `.harness/project/*`.
