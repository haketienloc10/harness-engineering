# Testing Policy

## Default verification

Chạy verification thật khi có thể:

```bash
bash .harness/scripts/verify.sh
```

`verify.sh` cố gắng detect stack phổ biến và chạy command có sẵn.

## Runtime smoke

Nếu app có UI hoặc API runtime:

```bash
bash .harness/scripts/smoke.sh
```

Với Vite:

```bash
APP_URL=http://localhost:5173 bash .harness/scripts/smoke.sh
```

## Evidence

Evaluator phải ghi command đã chạy và kết quả. Với UI task, static checks hoặc curl smoke không đủ nếu requirement là behaviour như validation, create/update/delete, filtering, navigation, state transition, persistence, error state, hoặc empty state.

Nếu thiếu evidence cho behaviour bắt buộc, verdict phải là `Fail`, `Needs Fix`, hoặc `Blocked`.
