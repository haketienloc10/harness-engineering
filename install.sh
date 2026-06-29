#!/usr/bin/env bash
set -euo pipefail

REPO_URL="${HARNESS_REPO_URL:-https://github.com/haketienloc10/harness-engineering}"
REF="${HARNESS_REF:-main}"
TARGET_DIR="${1:-$PWD}"

usage() {
  cat <<'EOF'
Usage:
  curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/install.sh?$(date +%s)" | bash
  curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/install.sh?$(date +%s)" | bash -s -- /path/to/target

Environment:
  HARNESS_REPO_URL       GitHub repository URL. Default: https://github.com/haketienloc10/harness-engineering
  HARNESS_REF            Branch or tag to install. Default: main
  HARNESS_CLI_BASE_URL   Override CLI release asset base URL.
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

need() {
  command -v "$1" >/dev/null 2>&1 || {
    printf 'Error: %s is required\n' "$1" >&2
    exit 1
  }
}

abs_path() {
  case "$1" in
    "~") printf '%s\n' "$HOME" ;;
    "~/"*) printf '%s/%s\n' "$HOME" "${1#~/}" ;;
    /*) printf '%s\n' "$1" ;;
    *) printf '%s/%s\n' "$PWD" "$1" ;;
  esac
}

detect_platform() {
  case "$(uname -s):$(uname -m)" in
    Darwin:arm64) printf 'macos-arm64' ;;
    Darwin:x86_64) printf 'macos-x64' ;;
    Linux:x86_64) printf 'linux-x64' ;;
    Linux:aarch64|Linux:arm64) printf 'linux-arm64' ;;
    *)
      printf 'unsupported'
      ;;
  esac
}

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print $1 }'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    return 1
  fi
}

backup_path() {
  local path="$1"
  [ -e "$path" ] || return 0
  mkdir -p "$BACKUP_DIR"
  mv "$path" "$BACKUP_DIR/$(basename "$path")"
  printf 'backup  %s -> %s\n' "$path" "$BACKUP_DIR/$(basename "$path")"
}

snapshot_path() {
  local path="$1"
  [ -e "$path" ] || return 0
  mkdir -p "$BACKUP_DIR"
  cp -R "$path" "$BACKUP_DIR/$(basename "$path")"
  printf 'backup  %s -> %s\n' "$path" "$BACKUP_DIR/$(basename "$path")"
}

copy_tree_files() {
  local src_dir="$1"
  local dest_dir="$2"
  local label="$3"
  local count=0

  [ -d "$src_dir" ] || return 0
  mkdir -p "$dest_dir"

  while IFS= read -r -d '' file; do
    local rel="${file#"$src_dir"/}"
    mkdir -p "$(dirname "$dest_dir/$rel")"
    cp "$file" "$dest_dir/$rel"
    count=$((count + 1))
  done < <(find "$src_dir" -type f -print0)

  printf 'update  %s (%s files)\n' "$label" "$count"
}

append_gitignore_rules() {
  local target="$TARGET_DIR/.gitignore"
  touch "$target"

  local changed=0
  for rule in \
    "harness.db" \
    "harness.db-wal" \
    "harness.db-shm" \
    ".harness-backup/" \
    ".agent-harness/bin/harness-cli.bin" \
    ".agent-harness/bin/harness-cli.exe" \
    "_harness/bin/harness-cli.bin" \
    "_harness/bin/harness-cli.exe"
  do
    if ! grep -Fxq "$rule" "$target"; then
      if [ "$changed" -eq 0 ]; then
        {
          [ -s "$target" ] && printf '\n'
          printf '# Harness local state\n'
        } >> "$target"
      fi
      printf '%s\n' "$rule" >> "$target"
      changed=1
    fi
  done

  [ "$changed" -eq 0 ] && printf 'skip    .gitignore\n' || printf 'update  .gitignore\n'
}

install_cli() {
  local platform tag base fallback_base name binary checksum expected actual
  platform="$(detect_platform)"
  if [ "$platform" = "unsupported" ]; then
    printf 'warn    CLI binary not installed: unsupported platform %s/%s\n' "$(uname -s)" "$(uname -m)" >&2
    return 0
  fi

  tag="$(awk 'NF && $1 !~ /^#/ { print $1; exit }' "$SRC/_harness/harness-cli-release-tag" 2>/dev/null || true)"
  [ -n "$tag" ] || tag="latest"

  if [ -n "${HARNESS_CLI_BASE_URL:-}" ]; then
    base="${HARNESS_CLI_BASE_URL%/}"
    fallback_base=""
  elif [ "$tag" = "latest" ]; then
    base="$REPO_URL/releases/latest/download"
    fallback_base="https://github.com/hoangnb24/repository-harness/releases/latest/download"
  else
    base="$REPO_URL/releases/download/$tag"
    fallback_base="https://github.com/hoangnb24/repository-harness/releases/download/$tag"
  fi

  name="harness-cli-$platform"
  binary="$TARGET_DIR/_harness/bin/harness-cli.bin"
  checksum="$TMP/$name.sha256"

  if ! curl -fsSL "$base/$name" -o "$binary" 2>/dev/null; then
    if [ -n "$fallback_base" ] && curl -fsSL "$fallback_base/$name" -o "$binary" 2>/dev/null; then
      base="$fallback_base"
    else
      printf 'warn    CLI binary not installed: could not download %s/%s\n' "$base" "$name" >&2
      rm -f "$binary"
      return 0
    fi
  fi
  chmod 755 "$binary"

  local checksum_status="unverified"
  if curl -fsSL "$base/$name.sha256" -o "$checksum" 2>/dev/null; then
    expected="$(awk '{ print $1; exit }' "$checksum")"
    actual="$(sha256 "$binary" || true)"
    if [ -n "$expected" ] && [ -n "$actual" ] && [ "$expected" != "$actual" ]; then
      rm -f "$binary"
      printf 'Error: checksum mismatch for %s\n' "$name" >&2
      exit 1
    fi
    checksum_status="checksum verified"
  fi

  if ! "$binary" --help >/dev/null 2>&1; then
    rm -f "$binary"
    printf 'warn    CLI binary not installed: downloaded binary cannot run on this system\n' >&2
    return 0
  fi
  printf 'install _harness/bin/harness-cli (%s, %s)\n' "$platform" "$checksum_status"
  CLI_INSTALLED=1
}

build_cli_from_source() {
  [ "$CLI_INSTALLED" -eq 0 ] || return 0

  local manifest binary source_name package_args
  package_args=""
  if [ -f "$SRC/Cargo.toml" ]; then
    manifest="$SRC/Cargo.toml"
    package_args="-p harness-cli"
  elif [ -f "$SRC/crates/harness-cli/Cargo.toml" ]; then
    manifest="$SRC/crates/harness-cli/Cargo.toml"
  else
    printf 'warn    CLI source not built: Cargo.toml not found at repo root or crates/harness-cli\n' >&2
    return 0
  fi

  if ! command -v cargo >/dev/null 2>&1; then
    printf 'warn    CLI source not built: cargo is not installed\n' >&2
    return 0
  fi

  printf 'build   harness-cli from %s\n' "${manifest#$SRC/}"
  if ! cargo build --release --manifest-path "$manifest" $package_args; then
    printf 'warn    CLI source build failed\n' >&2
    return 0
  fi

  source_name="harness-cli"
  binary="$(dirname "$manifest")/target/release/$source_name"
  if [ ! -x "$binary" ] && [ -x "$SRC/target/release/$source_name" ]; then
    binary="$SRC/target/release/$source_name"
  fi
  if [ ! -x "$binary" ]; then
    printf 'warn    CLI source built, but target/release/harness-cli was not found\n' >&2
    return 0
  fi

  cp "$binary" "$TARGET_DIR/_harness/bin/harness-cli.bin"
  chmod 755 "$TARGET_DIR/_harness/bin/harness-cli.bin"
  printf 'install _harness/bin/harness-cli (built from source)\n'
  CLI_INSTALLED=1
}

need curl
need tar

TARGET_DIR="$(abs_path "$TARGET_DIR")"
TMP="$(mktemp -d)"
BACKUP_DIR="$TARGET_DIR/.harness-backup/$(date +%Y%m%d%H%M%S)"
CLI_INSTALLED=0
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TARGET_DIR"

SCRIPT_PATH="${BASH_SOURCE[0]:-$0}"
SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_PATH")" 2>/dev/null && pwd -P || true)"

if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/AGENTS.md" ] && [ -d "$SCRIPT_DIR/_harness" ]; then
  SRC="$SCRIPT_DIR"
  printf 'source  %s\n' "$SRC"
else
  ARCHIVE="$REPO_URL/archive/refs/heads/$REF.tar.gz"
  if ! curl -fsSL "$ARCHIVE" | tar -xz -C "$TMP"; then
    ARCHIVE="$REPO_URL/archive/refs/tags/$REF.tar.gz"
    curl -fsSL "$ARCHIVE" | tar -xz -C "$TMP"
  fi

  SRC="$(find "$TMP" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
  [ -n "$SRC" ] || {
    printf 'Error: downloaded archive did not contain a source directory\n' >&2
    exit 1
  }
  printf 'source  %s#%s\n' "$REPO_URL" "$REF"
fi
printf 'target  %s\n' "$TARGET_DIR"

backup_path "$TARGET_DIR/AGENTS.md"
backup_path "$TARGET_DIR/.agent-harness"
backup_path "$TARGET_DIR/_harness"
snapshot_path "$TARGET_DIR/docs"

cp -R "$SRC/AGENTS.md" "$TARGET_DIR/AGENTS.md"
cp -R "$SRC/_harness" "$TARGET_DIR/_harness"
copy_tree_files "$SRC/docs" "$TARGET_DIR/docs" "docs/"
printf 'install AGENTS.md\n'
printf 'install _harness/\n'

append_gitignore_rules
install_cli
build_cli_from_source

printf '\nDone. Next:\n'
if [ "$CLI_INSTALLED" -eq 1 ]; then
  printf '  _harness/bin/harness-cli init\n'
  printf '  _harness/bin/harness-cli query matrix\n'
else
  printf '  publish a CLI release asset or set HARNESS_CLI_BASE_URL, then run _harness/bin/harness-cli init\n'
fi
