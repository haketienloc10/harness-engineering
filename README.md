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
- Không reset run history, backlog local, project adapter, hoặc codebase knowledge base khi update.

Harness có ba lớp:

1. **Artifact Protocol**: templates, run folders, evidence files, and `RUN_INDEX.md`.
2. **Role Policy**: Planner, Contract Reviewer, Generator, and Evaluator responsibilities.
3. **Lifecycle Orchestrator**: `run.yaml`, `run-manifest.md`, state machine, gates, template-based subagent spawning, and validation.

## Template-Based Subagent Orchestration

Harness core lifecycle roles MUST be executed by separate spawned subagents instantiated from fixed templates:

- `template/.harness/subagents/planner.md`
- `template/.harness/subagents/contract-reviewer.md`
- `template/.harness/subagents/generator.md`
- `template/.harness/subagents/evaluator.md`

```txt
Planner -> Contract Reviewer -> Generator -> Evaluator
```

The top-level agent acts as coordinator/orchestrator only. It may select the required role, load the role template, pass task-specific inputs, collect the role artifact, and decide the next workflow step based on that artifact.

The coordinator must not create free-form prompts for core lifecycle roles, execute those roles itself, modify role responsibilities, weaken evidence requirements, bypass role separation, edit source/tests/config, repair implementation failures directly, or continue when subagent spawning is unavailable.

If no spawned subagent runtime is available, the run is blocked before Planner execution. There is no degraded single-session fallback.

If Evaluator returns a non-passing result, the coordinator routes through a bounded Generator rework packet and spawns Generator again. If Generator cannot be spawned, stop with `BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE`.

Required blocked message:

```text
Subagent runtime unavailable.
Harness lifecycle requires template-based subagent orchestration.
This run is blocked.
No lifecycle role may be executed in this session.
```

Harness dùng một execution namespace:

- `.harness/runs/RUN-*`: normal runs cho từng implementation unit.
- `.harness/runs/EPIC-*`: Epic containers cho long-running tasks, giữ roadmap, acceptance matrix, decision log, và child run index.
- `.harness/runs/EPIC-*/runs/RUN-*`: child runs, là nơi chứa implementation contract, evaluator report, worklog, và final summary.

Trước khi tạo normal run, Harness phải classify request. Multi-phase, broad, MVP, full feature, core loop, complete playable, hoặc task không verify gọn trong một run phải thành Epic và được chia thành child runs nhỏ.

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

- `ask`: hỏi tương tác khi target đã có `AGENTS.md`.
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
    HARNESS_SKILLS.md
    guides/
    subagents/
    workflows/
    skills/
    templates/
    project-templates/
    project/
    codebase/
    scripts/
    backlog/
    runs/
```

Các vùng ownership-safe:

- `.harness/project/*` chỉ được tạo nếu chưa có. Đây là project adapter của repo đích.
- `.harness/codebase/*` chỉ được tạo nếu chưa có. Đây là source-navigation và change-impact cache do repo đích sở hữu; nó không thay thế hoặc duplicate `.harness/project/*`.
- `.harness/runs/*` không bị reset khi update. Thư mục này chứa normal runs, Epic containers, và child runs.
- Legacy `.harness/epics/*`, nếu có từ Harness cũ, không bị xóa khi update.
- `.harness/backlog/HARNESS_BACKLOG.md` không bị đè nếu đã tồn tại.
- `.harness/guides/*`, `.harness/templates/*`, `.harness/scripts/*`, `.harness/project-templates/*` là kernel/template layer có thể được update có chủ đích.
- `.harness/subagents/*` và `.harness/workflows/*` là kernel layer cho template-based subagent orchestration.
- `.harness/HARNESS_SKILLS.md` và seeded `.harness/skills/*` là Harness workflow skill layer được cài vào target repo; installer không xóa skill file local khác.

## Sau khi cài

After installation, ask your agent:

```txt
Read `.harness/HARNESS_SKILLS.md` and run the `project-sync` Harness workflow skill.
Then run `codebase-sync` if `.harness/codebase/*` is missing or stale.
```

Không cần cài native-agent skills.

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
    INSTALLATION.md
    HARNESS_SKILLS.md
    guides/
      HARNESS_PRINCIPLES.md
      AGENT_WORKFLOW.md
      RUN_CLASSIFICATION.md
      PROJECT_DISCOVERY.md
      LIFECYCLE_ORCHESTRATION.md
      SUBAGENT_EXECUTION.md
      PLANNING_AND_CONTRACTS.md
      TESTING_POLICY.md
      PARALLEL_WORK.md
      BACKLOG_POLICY.md
      LANGUAGE_POLICY.md
      LONG_TASK_POLICY.md
    subagents/
      planner.md
      contract-reviewer.md
      generator.md
      evaluator.md
    workflows/
      default-lifecycle.md
      epic-lifecycle.md
    skills/
      project-sync.md
      codebase-sync.md
    codebase/
      CODEBASE_INDEX.md
      CODEBASE_AREAS.md
      CODEBASE_ENTRYPOINTS.md
      CODEBASE_FLOWS.md
      CODEBASE_CHANGE_IMPACT.md
      CODEBASE_SOURCE_EVIDENCE.md
      CODEBASE_FRESHNESS.md
    templates/
      00-input.template.md
      01-planner-brief.template.md
      02-implementation-contract.template.md
      03-contract-review.template.md
      04-implementation-report.template.md
      05-evaluator-report.template.md
      06-fix-report.template.md
      generator-rework-packet.template.md
      07-final-summary.template.md
      00-epic-overview.template.md
      01-epic-roadmap.template.md
      02-epic-acceptance-matrix.template.md
      03-epic-contract-review.template.md
      03-epic-decision-log.template.md
      04-epic-run-index.template.md
      epic.yaml.template
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
      new-epic.sh
      new-run.sh
      list-epics.sh
      list-runs.sh
      link-run-to-epic.sh
      next-role.sh
      validate-epic.sh
      validate-run.sh
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
- chỉnh `.harness/codebase/*` theo source tree, entrypoints, flows, và change-impact thực tế;
- thêm rule local;
- thêm validation command;
- thêm backlog proposal;
- thay đổi guide/template/script nếu cần;
- lưu run history riêng.

Repo này không kiểm soát project đích sau khi cài.
