#!/usr/bin/env bash
set -Eeuo pipefail

REPO_OWNER="${HARNESS_LITE_OWNER:-haketienloc10}"
REPO_NAME="${HARNESS_LITE_REPO:-harness-engineering}"
# Giữ HARNESS_LITE_REF để tương thích các lệnh cũ. SOURCE_REF là tên rõ nghĩa
# hơn cho việc chọn branch/tag của payload cài đặt.
REF="${HARNESS_LITE_SOURCE_REF:-${HARNESS_LITE_REF:-main}}"
# Ưu tiên đường dẫn đối số để cài nhanh vào một repo local; biến môi trường và
# $PWD vẫn giữ nguyên hành vi cũ.
TARGET_DIR="${1:-${HARNESS_LITE_TARGET_DIR:-$PWD}}"
# `repository` giữ nguyên contract cài một repo. `coordination` dành cho Git
# root điều phối nhiều nested repository độc lập.
INSTALL_MODE="${HARNESS_INSTALL_MODE:-repository}"

ARCHIVE_URL="https://codeload.github.com/${REPO_OWNER}/${REPO_NAME}/tar.gz/${REF}"

# Payload tối thiểu để một repo đích vận hành Harness. Dùng allowlist thay vì
# copy cả thư mục rồi lọc: tài liệu, evidence và config của repo nguồn không
# được coi là một phần của runtime cài đặt.
INSTALL_ITEMS=(
  "_harness/bin/harness-cli"
  "_harness/workflow.toml"
  "_harness/command-manifest.txt"
  "_harness/tests/policy-parity-cases.toml"
  "_harness/templates"
  "_harness/scripts/schema"
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

validate_install_mode() {
  case "$INSTALL_MODE" in
    repository|coordination) ;;
    *) fail "HARNESS_INSTALL_MODE phải là repository hoặc coordination: $INSTALL_MODE" ;;
  esac
}

ensure_coordination_git_root() {
  [ "$INSTALL_MODE" = "coordination" ] || return 0
  command -v git >/dev/null 2>&1 || fail "coordination mode cần git để xác minh Git root"

  local target_root git_root
  target_root="$(cd "$TARGET_DIR" && pwd -P)"
  git_root="$(git -C "$TARGET_DIR" rev-parse --show-toplevel 2>/dev/null)" \
    || fail "coordination mode chỉ cài vào Git root: $TARGET_DIR"
  git_root="$(cd "$git_root" && pwd -P)"
  [ "$target_root" = "$git_root" ] \
    || fail "coordination mode chỉ cài vào Git root: $TARGET_DIR"
}

# Sinh block cài đặt từ AGENTS.md canonical. Phần shared phải byte-for-byte để
# repo đích luôn nhận instruction surface agents-first mới nhất.
build_harness_block() {
  local shared_source="$SRC_DIR/AGENTS.md"
  [ -f "$shared_source" ] || fail "Thiếu canonical AGENTS.md trong source archive"
  printf '%s\n' "$HARNESS_BLOCK_BEGIN"
  cat <<'EOF'

# Harness đã cài đặt

- Xem `_harness/` là runtime cho agent, không phải source sản phẩm của repo đích.
- Xem `docs/product/`, `docs/stories/`, và `docs/decisions/` là durable product
  record của repo đích.
- Xem các path khác là thuộc repo đích, trừ khi file tự chỉ định khác.
EOF
  build_harness_topology_block
  printf '%s\n' "$HARNESS_SHARED_BEGIN"
  cat "$shared_source"
  printf '%s\n' "$HARNESS_SHARED_END"
  printf '%s\n' "$HARNESS_BLOCK_END"
}

build_harness_topology_block() {
  case "$INSTALL_MODE" in
    repository)
      cat <<'EOF'

## Harness topology

- Installation mode: `repository`.
- Harness lifecycle applies to this repository only.
- A nested directory with its own `.git` is outside this installation unless it
  has its own separately installed Harness.
EOF
      ;;
    coordination)
      cat <<'EOF'

## Harness topology

- Installation mode: `coordination`.
- This directory is the coordination Git root. It owns cross-repository task
  lifecycle, traces, proofs, capsules, and records under `docs/`.
- A descendant directory with its own `.git` is an independent delivery
  repository. Its source, product docs, build, tests, commits, and release flow
  remain owned by that repository.
- Run `_harness/bin/harness-cli` only from this coordination root. Do not run it
  from a delivery repository and do not create `_harness/`, `.harness-id`, or
  `harness.db` there.
- Start and finish lifecycle work at this root. Run source commands in the
  affected delivery repository; make its path explicit when recording proof.
- Root `docs/` records cross-repository scope, contracts, decisions, and
  integration evidence. They do not replace records owned by a delivery
  repository.
EOF
      ;;
  esac
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
      printf 'Thêm instructions riêng của repo tại đây.\n\n'
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
    printf 'installation_mode = %s\n' "$INSTALL_MODE"
    printf '\n[files]\n'
    if [ "${#INSTALLED_FILES[@]}" -gt 0 ]; then
      printf '%s\n' "${INSTALLED_FILES[@]}" | LC_ALL=C sort
    fi
  } >"$manifest"
  log "Ghi manifest: _harness/.harness-manifest (${#INSTALLED_FILES[@]} file)"
}

write_installation_config() {
  local config="$TARGET_DIR/_harness/installation.toml"
  mkdir -p "$(dirname "$config")"
  cat >"$config" <<EOF
# Harness installation topology; generated by install.sh, do not edit.
version = 1
mode = "$INSTALL_MODE"
root_only = $([ "$INSTALL_MODE" = coordination ] && printf true || printf false)
EOF
  INSTALLED_FILES+=("_harness/installation.toml")
  log "Ghi installation config: _harness/installation.toml ($INSTALL_MODE)"
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

validate_install_mode

check_platform

[ -d "$TARGET_DIR" ] || fail "TARGET_DIR không tồn tại: $TARGET_DIR"
ensure_coordination_git_root

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
EXISTING_FILES=0

# Copy một file trong allowlist. _harness/ là payload do Harness sở hữu; các
# generated integration files ngoài nó vẫn được xử lý riêng và không ghi đè.
copy_file() {
  local rel="$1" src="$2"
  local dest="$TARGET_DIR/$rel"
  # _harness/ thuộc Harness hoàn toàn -> luôn ghi đè để nâng cấp runtime.
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
    # Duyệt từng file trong directory đã được allowlist.
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

# Nhúng block Harness vào AGENTS.md repo đích (sau khi _harness/ đã có mặt) và
# ghi manifest đánh dấu chế độ "đã cài Harness".
install_agents_md
install_gitignore_block
ensure_repository_id
write_installation_config
write_manifest

if [ "$EXISTING_FILES" -gt 0 ]; then
  log "Giữ nguyên $EXISTING_FILES file đã có sẵn của repo đích (không ghi đè)."
fi

if [ "${#MISSING_ITEMS[@]}" -gt 0 ]; then
  log "Một số item không tồn tại trong repo source:"
  for item in "${MISSING_ITEMS[@]}"; do
    printf '  - %s\n' "$item"
  done
fi

log "Hoàn tất. Payload đã cập nhật; harness.db không bị thay đổi. Chạy task start để ensure DB khi sẵn sàng."
