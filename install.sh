#!/usr/bin/env bash
set -Eeuo pipefail

REPO_OWNER="${HARNESS_LITE_OWNER:-haketienloc10}"
REPO_NAME="${HARNESS_LITE_REPO:-harness-engineering}"
REF="${HARNESS_LITE_REF:-main}"
TARGET_DIR="${HARNESS_LITE_TARGET_DIR:-$PWD}"

ARCHIVE_URL="https://codeload.github.com/${REPO_OWNER}/${REPO_NAME}/tar.gz/${REF}"

# Khung mẫu (scaffold) được cài vào repo đích. CHỈ liệt kê những thứ là bộ
# khung dùng chung cho mọi repo - KHÔNG liệt kê tài nguyên riêng của repo
# harness-engineering (xem EXCLUDE_PATHS bên dưới để lọc artifact lẫn trong các thư mục).
INSTALL_ITEMS=(
  ".editorconfig"
  ".prettierignore"
  ".prettierrc"
  "_harness"
  "docs"
  ".agents"
)

# AGENTS.md KHÔNG nằm trong INSTALL_ITEMS: thay vì copy nguyên file, ta NHÚNG
# block Harness vào AGENTS.md của repo đích (xem install_agents_md). Nhờ vậy nội
# dung "đây là tooling, KHÔNG phải source sản phẩm" chỉ xuất hiện ở repo ĐÍCH -
# còn harness-engineering (nơi _harness/ chính LÀ sản phẩm) không bị dính rule đó.
HARNESS_BLOCK_BEGIN="<!-- HARNESS:BEGIN -->"
HARNESS_BLOCK_END="<!-- HARNESS:END -->"

# Danh sách file thực sự được copy - ghi vào _harness/.harness-manifest ở cuối.
# Vừa là DẤU HIỆU "repo này đã cài Harness", vừa phục vụ gỡ/nâng cấp về sau.
INSTALLED_FILES=()

# Artifact là TÀI NGUYÊN riêng của harness-engineering - không phải khung mẫu,
# không được sao chép sang repo đích. So khớp theo đường dẫn tương đối tính từ gốc
# repo (xem is_excluded). Quy ước:
#   - "dir/*"      => bỏ MỌI file dưới dir đó
#   - "dir/keep/*" cộng nhánh keep ở is_excluded => giữ lại ngoại lệ
# Các thư mục product/stories/decisions/proposals chỉ giữ README/backlog/template
# generic; nội dung thực (story, decision record, proposal, read-model...) bị loại.
ensure_empty_dir() {
  # Một số thư mục scaffold (vd: proposals) sau khi lọc sẽ rỗng. Tạo sẵn để
  # agent có chỗ ghi mà không kéo theo artifact của repo nguồn.
  mkdir -p "$TARGET_DIR/_harness/docs/proposals"
}

# Trả về 0 (true => LOẠI) nếu path tương đối là artifact riêng của repo nguồn.
is_excluded() {
  local p="$1"
  case "$p" in
    # Dữ liệu vận hành / evidence / CSDL riêng của repo nguồn (đều trong _harness/)
    _harness/harness.db) return 0 ;;
    _harness/.harness-manifest) return 0 ;;
    _harness/evidence/*) return 0 ;;
    # Ma trận test được generate riêng cho repo nguồn
    _harness/docs/TEST_MATRIX.md) return 0 ;;
    # Bản đồ orient + wiki được generate riêng cho repo nguồn
    docs/KNOWLEDGE_INDEX.md) return 0 ;;
    docs/wiki/*) return 0 ;;
    # Thư mục scaffold: chỉ giữ hướng dẫn generic, bỏ nội dung thực của repo nguồn
    _harness/docs/proposals/*)
      case "$p" in _harness/docs/proposals/README.md) return 1 ;; *) return 0 ;; esac ;;
    docs/decisions/*)
      case "$p" in docs/decisions/README.md) return 1 ;; *) return 0 ;; esac ;;
    docs/product/*)
      case "$p" in docs/product/README.md) return 1 ;; *) return 0 ;; esac ;;
    docs/stories/epics/*)
      case "$p" in docs/stories/epics/README.md) return 1 ;; *) return 0 ;; esac ;;
    docs/stories/*)
      case "$p" in
        docs/stories/README.md | docs/stories/backlog.md) return 1 ;;
        *) return 0 ;;
      esac ;;
  esac
  return 1
}

# Sinh block Harness (kèm marker) để nhúng vào AGENTS.md của repo đích.
build_harness_block() {
  printf '%s\n' "$HARNESS_BLOCK_BEGIN"
  cat <<'EOF'

# Agent-First Harness

This repository has Harness installed.

`AGENTS.md` is the required agent entrypoint. Start here, then follow the files
listed below.

## Installed Surface Contract

- `_harness/` is the Harness operating scaffold for agents: CLI, runtime policy,
  schema migrations, templates, and workflow references. It is not target-repo
  product source. Do not treat it as application code unless the task is a
  Harness improvement.
- `docs/product/`, `docs/stories/`, and `docs/decisions/` are target-repo product
  contracts used by the Harness workflow. They are part of the target repo's
  durable product record, not Harness runtime internals.
- Everything outside `_harness/` belongs to the target repo unless a file
  explicitly says otherwise.

## Start Every Task

Read in order:

1. `AGENTS.md`
2. `_harness/HARNESS.md`
3. `_harness/FEATURE_INTAKE.md`
4. `_harness/CONTEXT_RULES.md`
5. `_harness/bin/harness-cli query matrix` when the CLI exists

Then read only what the lane and task require:

- `_harness/ARCHITECTURE.md` for structure, boundaries, data, providers,
  runtime, public contracts, or app surfaces.
- `_harness/TOOL_REGISTRY.md` before optional external tools.
- `docs/product/*` when product behavior changes.
- `docs/stories/*` when work maps to a story.
- `docs/decisions/*` when architecture, source hierarchy, durable records,
  validation, or high-risk behavior changes.
- `_harness/templates/*` before creating harness artifacts.

If `harness.db` is missing and the CLI exists, run:

```bash
_harness/bin/harness-cli init
```

If the CLI is unavailable, use markdown artifacts and record the missing CLI as
harness friction.

## Non-Negotiables

- Classify first: input type, risk flags, lane.
- Derive product truth from current user intent, product docs, stories,
  decisions, matrix proof, code, and tests.
- Convert specs into product docs, stories, decisions, and proof; do not grow a
  monolithic spec.
- Do not skip validation silently.
- Query capability before optional external tool use.
- Run dependent commands sequentially. Do not put a durable write and its
  follow-up read or verification in `multi_tool_use.parallel`.
- Ask humans only for real ambiguity, high-risk direction, credentials, paid or
  destructive actions, or explicit approval gates.
- Leave durable records for the next agent.

## Answer Quality

Avoid abstract answers.

When explaining decisions, plans, risks, bugs, architecture, or trade-offs, use concrete examples and step-by-step cause-and-effect reasoning.

Prefer this structure when useful:

1. What happens
2. Why it happens
3. Concrete example
4. Resulting impact
5. Recommended action

## Work Loop

1. Classify input type with `_harness/FEATURE_INTAKE.md`.
2. Run the risk checklist and choose `tiny`, `normal`, or `high-risk`.
3. Record intake when the CLI exists:
   `_harness/bin/harness-cli intake --type <type> --summary <text> --lane <lane>`.
4. Locate affected docs, stories, decisions, code, and tests.
5. Query proof matrix when the CLI exists.
6. Run `_harness/bin/harness-cli tool check` when the CLI exists.
7. Before optional external tools, run:
   `_harness/bin/harness-cli query tools --capability <capability> --status present`.
8. Implement the smallest safe slice for the lane.
9. Update product docs, story state, proof, decisions, templates, or backlog
   when the task changes them.
10. Validate for the lane.
11. Record a trace when the CLI exists.
12. Fix harness friction immediately or record backlog.

## Command Ordering

Parallelize only independent commands. Serialize every producer -> consumer
sequence.

Run these sequences in separate tool calls, in order:

- `harness-cli init` before any `harness-cli query ...`.
- `harness-cli story add/update/verify` before `harness-cli query matrix`.
- `harness-cli tool check` before `harness-cli query tools`.
- File edits before validation commands that read those files.
- Validation and durable story updates before `harness-cli trace`.

If violated, rerun the dependent read or validation sequentially and use only
the rerun result.

## Lanes

Tiny:

- Use for low-risk docs, copy, naming, narrow edits, or limited setup without
  schema, CRUD, auth, authorization, provider integration, or migrations.
- Record intake, patch directly, run quick checks, update changed docs.

Normal:

- Use for story-sized behavior with bounded blast radius.
- Create or update one story when behavior-bearing.
- Record proof with `story update`; store repeatable proof with `--verify` and
  run `story verify <id>` when available.
- Record a Standard trace.

High-risk:

- Use for security, data, scope, public contracts, multiple roles/platforms, or
  validation guarantees.
- Create a high-risk packet from `_harness/templates/high-risk-story/`.
- Read relevant decisions before implementation.
- Add a durable decision for meaningful behavior, architecture, authorization,
  data ownership, API shape, or validation changes.
- Record a Detailed trace.

Hard gates are high-risk unless the user explicitly narrows scope: auth,
authorization, data loss or migration, audit/security, external provider
behavior, or removing/weakening validation.

## Source Hierarchy

```text
Current user instruction
  -> docs/product/*
  -> docs/stories/*
  -> _harness/bin/harness-cli query matrix
  -> docs/decisions/* plus CLI decisions
  -> code and tests
  -> historical specs or examples
```

## Validation And Proof

Use the right checks or state the exact gap. For normal/high-risk story work:

```bash
_harness/bin/harness-cli story verify <story-id>
_harness/bin/harness-cli story update --id <story-id> --unit 1 --integration 1 --e2e 0 --platform 0 --evidence "<commands run>"
_harness/bin/harness-cli query matrix
```

Proof booleans use `1`/`0`, not `yes`/`no`.

## Trace And Final Response

Before the final response:

1. Re-check validation evidence.
2. Run `git status --short`.
3. Record a trace with `_harness/bin/harness-cli trace` when the CLI exists.
4. Confirm changed harness artifacts when relevant.
5. Confirm trace/friction status or name the gap.

Final response stays concise: changed surface, validation, durable records, and
remaining gap only.
EOF
  printf '%s\n' "$HARNESS_BLOCK_END"
}

# Thay nội dung giữa marker HARNESS:BEGIN/END bằng block mới (idempotent khi
# cài lại / nâng cấp). Block mới đã chứa sẵn cả hai marker.
replace_harness_block() {
  local file="$1" block="$2" tmp
  tmp="$(mktemp)"
  BLOCK="$block" awk -v b="$HARNESS_BLOCK_BEGIN" -v e="$HARNESS_BLOCK_END" '
    $0 == b { print ENVIRON["BLOCK"]; skip=1; next }
    $0 == e { skip=0; next }
    !skip   { print }
  ' "$file" > "$tmp"
  mv "$tmp" "$file"
}

# Đảm bảo AGENTS.md repo đích có block Harness, KHÔNG ghi đè hướng dẫn riêng của
# họ: tạo mới nếu chưa có; thay block nếu đã có marker; chèn cuối nếu chưa có.
install_agents_md() {
  local dest="$TARGET_DIR/AGENTS.md" block
  block="$(build_harness_block)"

  if [ ! -e "$dest" ]; then
    {
      printf '# Agent Instructions\n\n'
      printf 'Add project-specific agent instructions here.\n\n'
      printf '%s\n' "$block"
    } >"$dest"
    log "Tạo mới AGENTS.md + nhúng block Harness"
  elif grep -qF "$HARNESS_BLOCK_BEGIN" "$dest"; then
    replace_harness_block "$dest" "$block"
    log "Cập nhật block Harness trong AGENTS.md có sẵn"
  else
    printf '\n%s\n' "$block" >>"$dest"
    log "Chèn block Harness vào cuối AGENTS.md có sẵn"
  fi
}

# Ghi _harness/.harness-manifest: đánh dấu repo đã cài Harness + liệt kê file.
write_manifest() {
  local manifest="$TARGET_DIR/_harness/.harness-manifest"
  mkdir -p "$(dirname "$manifest")"
  {
    printf '# Harness manifest - sinh tự động bởi install.sh, KHÔNG sửa tay.\n'
    printf '# Sự hiện diện của file này = repo CÀI Harness (không phải repo nguồn).\n'
    printf 'source = %s/%s\n' "$REPO_OWNER" "$REPO_NAME"
    printf 'ref = %s\n' "$REF"
    printf '\n[files]\n'
    if [ "${#INSTALLED_FILES[@]}" -gt 0 ]; then
      printf '%s\n' "${INSTALLED_FILES[@]}" | LC_ALL=C sort
    fi
  } >"$manifest"
  log "Ghi manifest: _harness/.harness-manifest (${#INSTALLED_FILES[@]} file)"
}

log() {
  printf '[harness-engineering] %s\n' "$*"
}

fail() {
  printf '[harness-engineering] ERROR: %s\n' "$*" >&2
  exit 1
}

command -v curl >/dev/null 2>&1 || fail "Thiếu curl"
command -v tar >/dev/null 2>&1 || fail "Thiếu tar"

[ -d "$TARGET_DIR" ] || fail "TARGET_DIR không tồn tại: $TARGET_DIR"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

log "Đang tải ${REPO_OWNER}/${REPO_NAME}@${REF}..."
curl -fsSL "$ARCHIVE_URL" -o "$TMP_DIR/source.tar.gz"

log "Đang giải nén..."
tar -xzf "$TMP_DIR/source.tar.gz" -C "$TMP_DIR"

SRC_DIR="$(find "$TMP_DIR" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
[ -n "$SRC_DIR" ] || fail "Không tìm thấy thư mục source sau khi giải nén"

log "Cài khung mẫu vào workspace: $TARGET_DIR"

MISSING_ITEMS=()
SKIPPED_FILES=0
EXISTING_FILES=0

# Copy một file đơn lẻ, tôn trọng is_excluded + KHÔNG ghi đè file của repo đích.
copy_file() {
  local rel="$1" src="$2"
  if is_excluded "$rel"; then
    SKIPPED_FILES=$((SKIPPED_FILES + 1))
    return 0
  fi
  local dest="$TARGET_DIR/$rel"
  # _harness/ thuộc Harness hoàn toàn -> luôn ghi đè để NÂNG CẤP khung. Mọi path
  # khác (dotfile, docs/ workspace) có thể là tài sản của repo đích -> KHÔNG đè
  # nếu đã tồn tại, tránh nuốt config/nội dung sẵn có của họ.
  case "$rel" in
    _harness/*) : ;;
    *)
      if [ -e "$dest" ]; then
        EXISTING_FILES=$((EXISTING_FILES + 1))
        return 0
      fi
      ;;
  esac
  mkdir -p "$(dirname "$dest")"
  cp "$src" "$dest"
  INSTALLED_FILES+=("$rel")
}

for item in "${INSTALL_ITEMS[@]}"; do
  src="$SRC_DIR/$item"

  if [ ! -e "$src" ]; then
    MISSING_ITEMS+=("$item")
    continue
  fi

  if [ -d "$src" ]; then
    # Duyệt từng file, bỏ qua artifact theo is_excluded.
    while IFS= read -r -d '' f; do
      rel="${f#"$SRC_DIR"/}"
      copy_file "$rel" "$f"
    done < <(find "$src" -type f -print0)
    log "Copied dir: $item"
  else
    copy_file "$item" "$src"
    log "Copied file: $item"
  fi
done

ensure_empty_dir

# Nhúng block Harness vào AGENTS.md repo đích (sau khi _harness/ đã có mặt) và
# ghi manifest đánh dấu chế độ "đã cài Harness".
install_agents_md
write_manifest

if [ "$SKIPPED_FILES" -gt 0 ]; then
  log "Đã bỏ qua $SKIPPED_FILES file artifact (tài nguyên riêng của repo nguồn)."
fi

if [ "$EXISTING_FILES" -gt 0 ]; then
  log "Giữ nguyên $EXISTING_FILES file đã có sẵn của repo đích (không ghi đè)."
fi

if [ "${#MISSING_ITEMS[@]}" -gt 0 ]; then
  log "Một số item không tồn tại trong repo source:"
  for item in "${MISSING_ITEMS[@]}"; do
    printf '  - %s\n' "$item"
  done
fi

log "Hoàn tất."
