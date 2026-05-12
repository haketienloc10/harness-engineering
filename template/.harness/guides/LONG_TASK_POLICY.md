# Long Task / Epic Policy

## Mục đích

Harness dùng một execution namespace:

```txt
.harness/runs/
```

- `Run`: đơn vị thực thi nhỏ, có contract và verification riêng.
- `Epic`: run container để điều phối task dài hơi, giữ mục tiêu tổng, roadmap, decision log, acceptance matrix, và index của các child runs.

Epic không phải implementation run và không phải metadata folder độc lập. Chỉ child runs trong `EPIC-.../runs/` mới chứa implementation contract, generator worklog, evaluator report, fix report, và final summary.

## Layout

Normal run:

```txt
.harness/runs/RUN-YYYYMMDD-NNN-task-slug/
```

Epic container:

```txt
.harness/runs/EPIC-YYYYMMDD-NNN-task-slug/
  epic.yaml
  00-epic-overview.md
  01-roadmap.md
  02-acceptance-matrix.md
  03-decision-log.md
  04-run-index.md
  runs/
    RUN-001-child-task/
```

## Khi Nào Tạo Epic

Tạo Epic khi task thật sự cần nhiều run độc lập, ví dụ:

- nhiều milestone có thể verify riêng;
- nhiều user flow hoặc module tách biệt;
- cần nhiều vòng implementation/review;
- scope chưa thể đóng sạch trong một run;
- dài hơn một phiên làm việc;
- có dependency giữa các bước;
- có acceptance criteria cấp Epic và acceptance criteria cấp run.

Rule bắt buộc: agent không được tạo Epic nếu chỉ xác định được một concrete run. Nếu task đủ lớn để tạo Epic, planner phải định nghĩa ít nhất hai child runs có thể verify độc lập trước khi implementation bắt đầu.

## Khi Chỉ Tạo Normal Run

Dùng normal run nếu:

- chỉ biết một run cụ thể;
- scope nhỏ hoặc có thể verify trong một lần hợp lý;
- impacted files ít;
- acceptance criteria rõ;
- không cần roadmap hoặc dependency đáng kể.

Nếu chỉ biết một run nhưng nghi có follow-up, tạo normal run và ghi follow-up proposal/backlog thay vì tạo Epic rỗng hoặc Epic chỉ có một run.

## Cách Chia Epic Thành Child Runs

Mỗi child run phải có mục tiêu hẹp và evidence rõ:

- một milestone nhỏ;
- một user flow;
- một module hoặc lát thay đổi kỹ thuật;
- một migration hoặc adapter riêng;
- một vòng fix/evaluation tách biệt nếu rủi ro cao.

Tạo child run bằng:

```bash
bash .harness/scripts/new-run.sh --within EPIC-YYYYMMDD-NNN-task-slug "child task"
```

`--epic` có thể tồn tại như alias tương thích, nhưng docs và workflow mới phải dùng `--within`.

Mỗi child run vẫn giữ lifecycle multi-agent bình thường:

```txt
Planner Agent -> Contract Reviewer Agent -> Generator Agent -> Evaluator Agent -> Final Summary
```

Acceptance criteria cấp Epic được track trong `02-acceptance-matrix.md`; evidence phải trỏ tới child runs.

Với mỗi child run:

- Contract Reviewer Agent phải độc lập với Planner Agent của child run đó trong production mode.
- Evaluator Agent phải độc lập với Generator Agent của child run đó trong production mode.
- Epic coordinator không được thay thế Evaluator approval của child run.
- Nếu child run dùng fallback single-session, artifact phải mark `runtime_mode: fallback_single_session` và `independence: degraded`.

## Quy Tắc Không Tạo Run Khổng Lồ

Không tạo run có contract quá rộng, ví dụ:

- nhiều feature độc lập trong cùng một contract;
- nhiều UI flow không thể smoke test cùng lúc;
- nhiều module có dependency chưa rõ;
- verification phải chờ toàn bộ project hoàn tất;
- worklog/final summary dự kiến quá dài để resume an toàn.

Nếu contract bắt đầu phình to, dừng lại và chuyển task sang Epic chỉ khi xác định được ít nhất hai child runs. Nếu chưa xác định được, đóng phạm vi run hiện tại và ghi follow-up.

## Quy Tắc Resume Long Task

Khi resume task dài hơi:

1. Kiểm tra `.harness/runs/RUN_INDEX.md`.
2. Tìm Epic container liên quan dưới `.harness/runs/EPIC-*`.
3. Đọc `epic.yaml`, `00-epic-overview.md`, `01-roadmap.md`, `02-acceptance-matrix.md`, `03-decision-log.md`, và `04-run-index.md`.
4. Xác định child run gần nhất và trạng thái verification.
5. Chỉ tạo child run mới khi biết rõ milestone tiếp theo và dependency đã đủ.
6. Không sửa artifact cũ để che failure; tạo fix report hoặc child run mới tùy phạm vi.

Nếu user tiếp tục cùng một mục tiêu lớn, agent phải ưu tiên dùng Epic active hiện có thay vì tạo Epic trùng lặp.

## Quy Tắc Evaluator Cho Epic

Evaluator ở cấp run kiểm chứng acceptance criteria của run.

Evaluator ở cấp Epic kiểm tra:

- các required Epic acceptance criteria đã Pass;
- mỗi criteria có evidence từ child run liên quan;
- các child runs liên quan có evaluator report;
- decision log phản ánh quyết định lớn;
- không còn blocker mở;
- roadmap và `04-run-index.md` khớp thực tế.

Evaluator không được đóng Epic chỉ vì run cuối pass nếu acceptance matrix cấp Epic chưa đủ evidence.

Epic coordinator có thể tổng hợp roadmap, decision log, và acceptance matrix, nhưng không được substitute cho independent Evaluator approval ở cấp child run.

## Quy Tắc Đóng Epic

Chỉ đóng Epic khi:

- tất cả required acceptance criteria trong `02-acceptance-matrix.md` đã Pass;
- mọi required acceptance row có evidence từ child runs;
- các child runs liên quan đã có evaluator report;
- không còn blocker mở;
- decision log đã được cập nhật;
- final epic summary đã được viết hoặc `00-epic-overview.md` đã cập nhật trạng thái cuối;
- `epic.yaml`, `04-run-index.md`, và `.harness/runs/RUN_INDEX.md` được cập nhật status.

Legacy `.harness/epics/` từ Harness cũ không bị xóa khi update, nhưng workflow mới không được tạo Epic mới ở đó.
