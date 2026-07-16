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
HARNESS_SHARED_BEGIN="<!-- HARNESS:SHARED:BEGIN -->"
HARNESS_SHARED_END="<!-- HARNESS:SHARED:END -->"
HARNESS_IGNORE_BEGIN="# HARNESS:BEGIN local state"
HARNESS_IGNORE_END="# HARNESS:END local state"

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
    # Runtime policy is now command/config-backed. Retain source-only records
    # in harness-engineering but do not install them into target repositories.
    _harness/HARNESS.md|_harness/FEATURE_INTAKE.md|_harness/CONTEXT_RULES.md|_harness/TRACE_SPEC.md|_harness/TOOL_REGISTRY.md|_harness/TEST_MATRIX.md|_harness/ARCHITECTURE.md|_harness/IMPROVEMENT_PROTOCOL.md|_harness/HARNESS_AUDIT.md|_harness/HARNESS_COMPONENTS.md|_harness/HARNESS_MATURITY.md|_harness/README.md) return 0 ;;
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

# Sinh block Harness từ root AGENTS.md canonical của source repository. Phần
# Installed Surface Contract chỉ thuộc target; phần nằm giữa HARNESS:SHARED là
# byte-for-byte từ AGENTS.md và được parity-test.
build_harness_block() {
  local shared_source="$SRC_DIR/AGENTS.md"
  [ -f "$shared_source" ] || fail "Thiếu canonical AGENTS.md trong source archive"
  printf '%s\n' "$HARNESS_BLOCK_BEGIN"
  cat <<'EOF'

# Installed Harness Surface

This repository has Harness installed.

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
EOF
  printf '%s\n' "$HARNESS_SHARED_BEGIN"
  cat "$shared_source"
  printf '%s\n' "$HARNESS_SHARED_END"
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

build_harness_ignore_block() {
  printf '%s\n' "$HARNESS_IGNORE_BEGIN"
  cat <<'EOF'
# Harness local operational state; generated by install.sh.
harness.db
harness.db-wal
harness.db-shm
harness.db.backups/
.harness-evidence/
docs/tasks/.staging/
EOF
  printf '%s\n' "$HARNESS_IGNORE_END"
}

install_gitignore_block() {
  local dest="$TARGET_DIR/.gitignore" block tmp
  block="$(build_harness_ignore_block)"
  if [ ! -e "$dest" ]; then
    printf '%s\n' "$block" >"$dest"
    log "Tạo .gitignore với block Harness local-state"
  elif grep -qF "$HARNESS_IGNORE_BEGIN" "$dest"; then
    tmp="$(mktemp)"
    BLOCK="$block" awk -v b="$HARNESS_IGNORE_BEGIN" -v e="$HARNESS_IGNORE_END" '
      $0 == b { print ENVIRON["BLOCK"]; skip=1; next }
      $0 == e { skip=0; next }
      !skip { print }
    ' "$dest" >"$tmp"
    mv "$tmp" "$dest"
    log "Cập nhật block Harness local-state trong .gitignore"
  else
    printf '\n%s\n' "$block" >>"$dest"
    log "Chèn block Harness local-state vào .gitignore hiện có"
  fi
}

generate_repository_id() {
  if command -v uuidgen >/dev/null 2>&1; then
    uuidgen | tr '[:upper:]' '[:lower:]'
  elif command -v openssl >/dev/null 2>&1; then
    openssl rand -hex 16 | sed 's/^\(........\)\(....\)\(....\)\(....\)\(............\)$/\1-\2-\3-\4-\5/'
  else
    fail "Không thể tạo .harness-id: cần uuidgen hoặc openssl"
  fi
}

ensure_repository_id() {
  local path="$TARGET_DIR/.harness-id"
  if [ -e "$path" ]; then
    log "Giữ nguyên .harness-id hiện có"
    return
  fi
  generate_repository_id >"$path"
  log "Tạo .harness-id; hãy commit file này để clone/worktree dùng cùng repository identity"
}

check_platform() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os/$arch" in
    Linux/x86_64) ;;
    *) fail "Không có harness-cli binary cho $os/$arch. Cài package phù hợp hoặc build harness-cli từ source trước khi chạy installer." ;;
  esac
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

check_platform

[ -d "$TARGET_DIR" ] || fail "TARGET_DIR không tồn tại: $TARGET_DIR"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

log "Đang tải ${REPO_OWNER}/${REPO_NAME}@${REF}..."
curl -fsSL "$ARCHIVE_URL" -o "$TMP_DIR/source.tar.gz"

log "Đang giải nén..."
tar -xzf "$TMP_DIR/source.tar.gz" -C "$TMP_DIR"

SRC_DIR="$(find "$TMP_DIR" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
[ -n "$SRC_DIR" ] || fail "Không tìm thấy thư mục source sau khi giải nén"

# Fail before mutating the target when the selected release archive cannot
# supply the only supported CLI payload for this platform. A partial Harness
# install without its command-first entrypoint is not a usable installation.
[ -x "$SRC_DIR/_harness/bin/harness-cli" ] \
  || fail "Thiếu executable _harness/bin/harness-cli trong source archive"

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
install_gitignore_block
ensure_repository_id
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

log "Hoàn tất. Payload đã cập nhật; harness.db không bị thay đổi. Chạy task start hoặc init để ensure DB khi sẵn sàng."
