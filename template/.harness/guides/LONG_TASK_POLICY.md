# Long Task / Epic Policy

## Mục đích

Harness dùng hai tầng để xử lý task dài hơi:

- `Run`: đơn vị thực thi nhỏ, có contract và verification riêng.
- `Epic`: đơn vị điều phối dài hơi, giữ mục tiêu tổng, milestone, dependency, decision log, acceptance matrix, và run index.

Không ép một task lớn vào một run khổng lồ. Long task phải được chia thành nhiều run nhỏ có thể verify độc lập.

## Khi Nào Phải Tạo Epic

Agent phải tạo Epic trước khi tạo run nếu task có một trong các dấu hiệu:

- cần nhiều hơn 1 milestone;
- có nhiều user flow;
- liên quan nhiều module;
- cần nhiều vòng implement/review;
- scope chưa thể đóng trong một run;
- dài hơn một phiên làm việc;
- cần dependency giữa các bước;
- có acceptance criteria cấp cao và acceptance criteria cấp run;
- không thể verify trong một lần hợp lý.

Rule bắt buộc: nếu task có nhiều milestone, nhiều feature, nhiều màn hình, nhiều module, hoặc không thể verify trong một lần hợp lý, agent không được tạo một run khổng lồ. Agent phải tạo Epic trước, sau đó chia thành nhiều run nhỏ.

## Khi Nào Chỉ Cần Run Đơn

Dùng một run đơn nếu:

- scope nhỏ;
- impacted files ít;
- acceptance criteria rõ;
- verification có thể hoàn thành ngay;
- không cần roadmap hoặc milestone;
- không có dependency đáng kể giữa nhiều bước.

## Cách Chia Epic Thành Run Nhỏ

Mỗi run trong Epic phải có mục tiêu hẹp và evidence rõ:

- một milestone nhỏ;
- một user flow;
- một module hoặc một lát thay đổi kỹ thuật;
- một migration hoặc adapter riêng;
- một vòng fix/evaluation tách biệt nếu rủi ro cao.

Mỗi run vẫn giữ lifecycle bình thường:

```txt
Planner -> Contract -> Evaluator -> Implementation -> Verification -> Summary
```

Mỗi run phải có acceptance criteria cấp run. Acceptance criteria cấp Epic được track trong `02-acceptance-matrix.md`.

## Quy Tắc Không Tạo Run Khổng Lồ

Không tạo run có contract quá rộng, ví dụ:

- nhiều feature độc lập trong cùng một contract;
- nhiều UI flow không thể smoke test cùng lúc;
- nhiều module có dependency chưa rõ;
- verification phải chờ toàn bộ project hoàn tất;
- worklog/final summary dự kiến quá dài để resume an toàn.

Nếu contract bắt đầu phình to, dừng lại và chuyển task sang Epic hoặc tách run tiếp theo trong Epic hiện có.

## Quy Tắc Resume Long Task

Khi resume task dài hơi:

1. Kiểm tra `.harness/epics/EPIC_INDEX.md`.
2. Đọc Epic active liên quan: `epic.yaml`, `00-epic-overview.md`, `01-roadmap.md`, `02-acceptance-matrix.md`, `03-decision-log.md`, `04-run-index.md`.
3. Xác định run gần nhất và trạng thái verification.
4. Chỉ tạo run mới khi biết rõ milestone tiếp theo và dependency đã đủ.
5. Không sửa artifact cũ để che failure; tạo fix report hoặc run mới tùy phạm vi.

Nếu user tiếp tục cùng một mục tiêu lớn, agent phải ưu tiên dùng Epic active hiện có thay vì tạo Epic trùng lặp.

## Quy Tắc Evaluator Cho Epic

Evaluator ở cấp run kiểm chứng acceptance criteria của run.

Evaluator ở cấp Epic kiểm tra:

- các required Epic acceptance criteria đã Pass;
- mỗi criteria có evidence từ run liên quan;
- các run liên quan có evaluator report;
- decision log phản ánh quyết định lớn;
- không còn blocker mở;
- roadmap và run index khớp thực tế.

Evaluator không được đóng Epic chỉ vì run cuối pass nếu acceptance matrix cấp Epic chưa đủ evidence.

## Quy Tắc Đóng Epic

Chỉ đóng Epic khi:

- tất cả required acceptance criteria trong `02-acceptance-matrix.md` đã Pass;
- các run liên quan đã có evaluator report;
- không còn blocker mở;
- decision log đã được cập nhật;
- final epic summary đã được viết hoặc `00-epic-overview.md` đã cập nhật trạng thái cuối;
- `epic.yaml` và `EPIC_INDEX.md` được cập nhật status.

Nếu scope thay đổi lớn, ghi quyết định vào `03-decision-log.md` trước khi tạo run tiếp theo.
