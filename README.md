# harness-engineering

`harness-engineering` là **Harness Seed Architecture Repository**.

Repo này không phải application source code và cũng không phải harness cứng cho một project cụ thể. Repo này chứa một **template kiến trúc** để cài một `.harness/` độc lập vào bất kỳ repository nào.

Sau khi cài đặt, repository đích sở hữu `.harness/` của chính nó và có thể tự phát triển tiếp mà không phụ thuộc repo này.

```txt
harness-engineering/
  README.md
  AGENTS.md                     # instruction cho chính repo seed này
  scripts/install-harness.sh     # bootstrap installer qua curl
  template/
    AGENTS.md                   # AGENTS.md được cài vào repo đích
    .harness/                   # template được copy thành target-repo/.harness/
```

## Nguyên tắc kiến trúc

```txt
Install = copy seed
Ownership = target repository
Update = optional, explicit, ownership-safe merge
```

Điều đó có nghĩa là:

- `template/.harness/` là nguồn seed.
- `target-repo/.harness/` là harness đã cài, thuộc quyền sở hữu của target repo.
- Không tự động đồng bộ ngược từ target repo về repo này.
- Không coi `.harness/` trong target repo là application source code.
- Không reset run history, backlog local, hoặc project adapter khi update.

## Cài nhanh bằng curl

Chạy từ thư mục repository đích:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/scripts/install-harness.sh?$(date +%s)" \
  | bash -s -- --target "$(pwd)" --agents-mode merge
```

Nếu repo đích đã có `AGENTS.md` và muốn giữ nguyên để tự merge:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/scripts/install-harness.sh?$(date +%s)" \
  | bash -s -- --target "$(pwd)" --agents-mode preserve
```

## Cài từ checkout

```bash
bash template/.harness/scripts/install.sh --target /path/to/your-repo --agents-mode merge
```

Preview trước khi ghi file:

```bash
bash template/.harness/scripts/install.sh --target /path/to/your-repo --agents-mode merge --dry-run
```

Không hỏi lại:

```bash
bash template/.harness/scripts/install.sh --target /path/to/your-repo --agents-mode merge --yes
```

## Mode cho AGENTS.md

Installer không ghi đè `AGENTS.md` im lặng. Chọn một mode:

- `merge`: backup file cũ, rồi tạo `AGENTS.md` hợp nhất. Đây là mode khuyến nghị để Harness được agent đọc tự động.
- `preserve`: giữ nguyên `AGENTS.md`, ghi Harness vào `AGENTS.harness.md`.
- `replace`: backup `AGENTS.md` hiện có, rồi thay bằng Harness `AGENTS.md`.
- `backup`: giống `replace`, nhưng thể hiện rõ ý định backup trước khi cài.

Nếu dùng `--yes` mà không truyền `--agents-mode`, khi target đã có `AGENTS.md`, script chọn `merge` để Harness được kích hoạt tự động.

## Nội dung được cài vào target repo

```txt
target-repo/
  AGENTS.md hoặc AGENTS.harness.md
  .harness/
    README.md
    INSTALLATION.md
    guides/
    templates/
    project-templates/
    project/
    scripts/
    backlog/
    runs/
```

Các vùng ownership-safe:

- `.harness/project/*` chỉ được tạo nếu chưa có. Đây là project adapter của repo đích.
- `.harness/runs/*` không bị reset khi update.
- `.harness/backlog/HARNESS_BACKLOG.md` không bị đè nếu đã tồn tại.
- `.harness/guides/*`, `.harness/templates/*`, `.harness/scripts/*`, `.harness/project-templates/*` là kernel/template layer có thể được update có chủ đích.

## Sau khi cài

```bash
bash .harness/scripts/inspect-project.sh
bash .harness/scripts/verify.sh
```

Nếu app có runtime UI hoặc API:

```bash
APP_URL=http://localhost:5173 bash .harness/scripts/smoke.sh
```

## Cấu trúc template

```txt
template/
  AGENTS.md
  .harness/
    README.md
    guides/
      HARNESS_PRINCIPLES.md
      AGENT_WORKFLOW.md
      PROJECT_DISCOVERY.md
      PLANNING_AND_CONTRACTS.md
      TESTING_POLICY.md
      PARALLEL_WORK.md
      BACKLOG_POLICY.md
      LANGUAGE_POLICY.md
    templates/
      00-input.template.md
      01-planner-brief.template.md
      02-implementation-contract.template.md
      03-evaluator-contract-review.template.md
      04-generator-worklog.template.md
      05-evaluator-report.template.md
      06-fix-report.template.md
      07-final-summary.template.md
      run.yaml.template
    project-templates/
      PROJECT_MAP.template.md
      SOURCE_OF_TRUTH.template.md
      STACK_PROFILE.template.md
      VALIDATION_PROFILE.template.md
      MODULE_MAP.template.md
      LOCAL_DECISIONS.template.md
    scripts/
      install.sh
      inspect-project.sh
      new-run.sh
      list-runs.sh
      check-conflicts.sh
      verify.sh
      smoke.sh
    backlog/
      HARNESS_BACKLOG.md
    runs/
      .gitkeep
```

## Vai trò của repo này

Repo này chỉ cung cấp seed architecture. Khi một project đã được cài Harness, project đó có quyền:

- chỉnh `.harness/project/*` theo thực tế project;
- thêm rule local;
- thêm validation command;
- thêm backlog proposal;
- thay đổi guide/template/script nếu cần;
- lưu run history riêng.

Repo này không kiểm soát project đích sau khi cài.
