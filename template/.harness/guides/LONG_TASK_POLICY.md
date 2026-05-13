# Long Task / Epic Policy

## Mục đích

Harness dùng một execution namespace:

```txt
.harness/runs/
```

- `Run`: đơn vị thực thi nhỏ, có contract và verification riêng.
- `Epic`: run container để điều phối task dài hơi, giữ mục tiêu tổng, roadmap, decision log, acceptance matrix, và index của các child runs.

Epic không phải implementation run và không phải metadata folder độc lập. Chỉ child runs trong `EPIC-.../runs/` mới chứa implementation contract, contract review, implementation report, evaluator report, và final summary.

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
  03-epic-contract-review.md
  04-run-index.md
  05-decision-log.md
  runs/
    RUN-001-child-task/
```

## Khi Nào Tạo Epic

Epic là bắt buộc khi task có bất kỳ tín hiệu long-task hoặc multi-phase nào. Một task tên hoặc request có dạng `phase 1-4`, `part 1-4`, `core loop`, `complete playable`, `full feature`, `end-to-end`, `MVP`, `large task`, `long task`, `toan-bo`, `hoan-thien`, `day-du`, `nhieu-phan-he`, `nhieu-luong`, hoặc `tu-dau-toi-cuoi` không được tạo thành một normal run.

Tạo Epic khi task thật sự cần nhiều run độc lập, ví dụ:

- nhiều phase hoặc part;
- nhiều milestone có thể verify riêng;
- nhiều user flow hoặc module tách biệt;
- cần nhiều vòng implementation/review;
- scope chưa thể đóng sạch trong một run;
- dài hơn một phiên làm việc;
- có dependency giữa các bước;
- có acceptance criteria cấp Epic và acceptance criteria cấp run;
- implementation cần nhiều checkpoint có thể verify độc lập.

Rule bắt buộc: nếu task đủ lớn để tạo Epic, Planner phải định nghĩa ít nhất hai child runs có thể verify độc lập trước khi implementation bắt đầu. Nếu chỉ biết một child run, Planner phải giảm phạm vi request xuống một normal run thật sự nhỏ, hoặc hỏi/derive decomposition bổ sung trước khi implementation. Không được dùng một normal run khổng lồ để thay thế Epic.

## Khi Chỉ Tạo Normal Run

Dùng normal run nếu:

- chỉ biết một run cụ thể;
- scope nhỏ hoặc có thể verify trong một lần hợp lý;
- impacted files ít;
- acceptance criteria rõ;
- không cần roadmap hoặc dependency đáng kể.

Nếu chỉ biết một run nhưng request không có tín hiệu Epic và scope thật sự nhỏ, tạo normal run và ghi follow-up proposal/backlog thay vì tạo Epic rỗng hoặc Epic chỉ có một run.

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

Mỗi child run vẫn giữ lifecycle Codex project-scoped subagent orchestration:

```txt
Planner -> Contract Reviewer -> Generator -> Evaluator -> Final Summary
```

Acceptance criteria cấp Epic được track trong `02-acceptance-matrix.md`; evidence phải trỏ tới child runs.

Với mỗi child run:

- Contract Reviewer subagent phải độc lập với Planner subagent của child run đó.
- Evaluator subagent phải độc lập với Generator subagent của child run đó.
- Epic coordinator không được thay thế Evaluator approval của child run.
- Không có single-session fallback cho child runs.

## Quy Tắc Không Tạo Run Khổng Lồ

Không tạo run có contract quá rộng, ví dụ:

- nhiều feature độc lập trong cùng một contract;
- nhiều UI flow không thể smoke test cùng lúc;
- nhiều module có dependency chưa rõ;
- verification phải chờ toàn bộ project hoàn tất;
- implementation report/final summary dự kiến quá dài để resume an toàn.

Nếu contract bắt đầu phình to, dừng lại và chuyển task sang Epic chỉ khi xác định được ít nhất hai child runs. Nếu chưa xác định được, đóng phạm vi run hiện tại và ghi follow-up.

Nếu một normal run đã được tạo nhưng sau đó phát hiện là oversized, mark run đó `SUPERSEDED_BY_EPIC`, không implement trong run đó, tạo Epic mới, và tiếp tục chỉ qua child runs.

Status block bắt buộc:

```md
# Run Status

- Status: SUPERSEDED_BY_EPIC
- Reason: <why this task is too large for a normal run>
- Superseded by: <EPIC-ID>
- Next action: continue only through Epic child runs
```

## Ví Dụ Sai / Đúng

Invalid normal run:

```txt
RUN-YYYYMMDD-NNN-develop-playable-ddm-core-loop-phase-1-4
```

Correct Epic:

```txt
EPIC-YYYYMMDD-NNN-develop-playable-ddm-core-loop/
  runs/RUN-001-core-board-state/
  runs/RUN-002-dice-summon-placement/
  runs/RUN-003-movement-combat/
  runs/RUN-004-ai-win-condition-polish/
```

Request:

> Develop playable DDM core loop phase 1-4

Wrong:

```txt
RUN-20260512-002-develop-playable-ddm-core-loop-phase-1-4
```

Reason:

- multiple phases;
- broad game loop;
- movement, dice, summon, combat, AI, and win condition likely need separate verification.

Correct:

```txt
EPIC-20260512-002-develop-playable-ddm-core-loop/
  runs/RUN-001-core-board-state-and-turn-shell/
  runs/RUN-002-dice-roll-summon-and-placement/
  runs/RUN-003-movement-terrain-and-combat/
  runs/RUN-004-ai-player-win-condition-and-playability/
```

## Quy Tắc Resume Long Task

Khi resume task dài hơi:

1. Kiểm tra `.harness/runs/RUN_INDEX.md`.
2. Tìm Epic container liên quan dưới `.harness/runs/EPIC-*`.
3. Đọc `epic.yaml`, `00-epic-overview.md`, `01-roadmap.md`, `02-acceptance-matrix.md`, `03-epic-contract-review.md`, `04-run-index.md`, và `05-decision-log.md`.
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
- `05-decision-log.md` đã được cập nhật;
- final epic summary đã được viết hoặc `00-epic-overview.md` đã cập nhật trạng thái cuối;
- `epic.yaml`, `04-run-index.md`, và `.harness/runs/RUN_INDEX.md` được cập nhật status.

Legacy `.harness/epics/` từ Harness cũ không bị xóa khi update, nhưng workflow mới không được tạo Epic mới ở đó.
