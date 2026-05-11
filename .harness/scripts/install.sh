#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TARGET_DIR="$(pwd)"
AGENTS_MODE="ask"
YES=0
FORCE=0

usage() {
  cat <<'EOF'
Usage:
  bash .harness/scripts/install.sh [options]

Install this Harness template into another repository.

Options:
  --target PATH              Repository to install into. Defaults to current directory.
  --agents-mode MODE         How to handle an existing AGENTS.md:
                               ask       Prompt interactively. Default.
                               merge     Backup existing file, then create a combined AGENTS.md.
                               preserve  Keep existing AGENTS.md and write AGENTS.harness.md.
                               replace   Backup existing file, then replace it with Harness AGENTS.md.
                               backup    Same install result as replace, with explicit backup.
  --yes, -y                  Do not prompt. If AGENTS.md exists and mode is ask, uses preserve.
  --force                    Allow updating an existing .harness/ directory without prompting.
  --help, -h                 Show this help.

Examples:
  bash .harness/scripts/install.sh --target /path/to/repo
  bash .harness/scripts/install.sh --target /path/to/repo --agents-mode merge
  bash .harness/scripts/install.sh --target /path/to/repo --agents-mode preserve --yes
EOF
}

die() {
  echo "Error: $*" >&2
  exit 1
}

info() {
  echo "==> $*"
}

backup_file() {
  local file="$1"
  local stamp
  stamp="$(date +%Y%m%d%H%M%S)"
  local backup="${file}.backup-${stamp}"

  if [ -e "$backup" ]; then
    backup="${backup}-$$"
  fi

  cp "$file" "$backup"
  printf "%s" "$backup"
}

copy_file() {
  local source="$1"
  local dest="$2"

  mkdir -p "$(dirname "$dest")"
  cp "$source" "$dest"
}

copy_dir_replace() {
  local source="$1"
  local dest="$2"

  rm -rf "$dest"
  mkdir -p "$(dirname "$dest")"
  cp -R "$source" "$dest"
}

write_clean_run_index() {
  local dest="$1"

  mkdir -p "$(dirname "$dest")"
  cat > "$dest" <<'EOF'
# Harness Run Index

| Run ID | Task | Status | Branch | Worktree | Owner | Started At | Last Updated |
|---|---|---|---|---|---|---|---|

## Status Values

- created
- planning
- contracting
- contract_review
- implementing
- evaluating
- fixing
- completed
- blocked
- cancelled
EOF
}

write_merged_agents() {
  local source_agents="$1"
  local existing_agents="$2"
  local dest_agents="$3"

  {
    cat "$source_agents"
    cat <<'EOF'

---

# Existing Repository Instructions

The instructions below were already present before Harness was installed. Keep them unless they conflict with the Harness bootstrap rules above.

EOF
    cat "$existing_agents"
  } > "$dest_agents"
}

prompt_agents_mode() {
  local choice

  cat <<'EOF'
AGENTS.md already exists in the target repository.

Choose how to continue:
  1) merge     Backup existing file, then combine Harness + existing instructions.
  2) preserve  Keep existing AGENTS.md and write Harness instructions to AGENTS.harness.md.
  3) replace   Backup existing file, then replace it with Harness AGENTS.md.
  4) backup    Backup existing file, then install Harness AGENTS.md.
EOF

  printf "Select [1-4] (default: 2): "
  read -r choice

  case "${choice:-2}" in
    1|merge) AGENTS_MODE="merge" ;;
    2|preserve) AGENTS_MODE="preserve" ;;
    3|replace) AGENTS_MODE="replace" ;;
    4|backup) AGENTS_MODE="backup" ;;
    *) die "Invalid AGENTS.md choice: $choice" ;;
  esac
}

install_agents() {
  local source_agents="$SOURCE_ROOT/AGENTS.md"
  local target_agents="$TARGET_DIR/AGENTS.md"
  local harness_agents="$TARGET_DIR/AGENTS.harness.md"
  local backup

  [ -f "$source_agents" ] || die "Source AGENTS.md not found: $source_agents"

  if [ ! -e "$target_agents" ]; then
    copy_file "$source_agents" "$target_agents"
    info "Installed AGENTS.md"
    return
  fi

  if [ "$AGENTS_MODE" = "ask" ] && [ "$YES" -eq 1 ]; then
    AGENTS_MODE="preserve"
  fi

  if [ "$AGENTS_MODE" = "ask" ]; then
    prompt_agents_mode
  fi

  case "$AGENTS_MODE" in
    merge)
      backup="$(backup_file "$target_agents")"
      write_merged_agents "$source_agents" "$backup" "$target_agents"
      info "Merged AGENTS.md and backed up the original to $backup"
      ;;
    preserve)
      if [ -e "$harness_agents" ]; then
        backup="$(backup_file "$harness_agents")"
        info "Backed up existing AGENTS.harness.md to $backup"
      fi
      copy_file "$source_agents" "$harness_agents"
      info "Preserved AGENTS.md and wrote Harness instructions to AGENTS.harness.md"
      ;;
    replace|backup)
      backup="$(backup_file "$target_agents")"
      copy_file "$source_agents" "$target_agents"
      info "Installed Harness AGENTS.md and backed up the original to $backup"
      ;;
    *)
      die "Unsupported --agents-mode: $AGENTS_MODE"
      ;;
  esac
}

install_harness_tree() {
  local target_harness="$TARGET_DIR/.harness"

  if [ -d "$target_harness" ] && [ "$FORCE" -ne 1 ] && [ "$YES" -ne 1 ]; then
    printf ".harness/ already exists in target. Update it? [y/N]: "
    local answer
    read -r answer
    case "$answer" in
      y|Y|yes|YES) ;;
      *) die "Cancelled because .harness/ already exists. Re-run with --force to update." ;;
    esac
  fi

  mkdir -p "$target_harness"
  copy_dir_replace "$SOURCE_ROOT/.harness/guides" "$target_harness/guides"
  copy_dir_replace "$SOURCE_ROOT/.harness/templates" "$target_harness/templates"
  copy_dir_replace "$SOURCE_ROOT/.harness/scripts" "$target_harness/scripts"

  mkdir -p "$target_harness/backlog"
  if [ -f "$SOURCE_ROOT/.harness/backlog/HARNESS_BACKLOG.md" ] && [ ! -f "$target_harness/backlog/HARNESS_BACKLOG.md" ]; then
    copy_file "$SOURCE_ROOT/.harness/backlog/HARNESS_BACKLOG.md" "$target_harness/backlog/HARNESS_BACKLOG.md"
  fi

  mkdir -p "$target_harness/runs"
  write_clean_run_index "$target_harness/runs/RUN_INDEX.md"
  info "Installed .harness/ workflow files"
}

parse_args() {
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --target)
        [ "$#" -ge 2 ] || die "--target requires a path"
        TARGET_DIR="$2"
        shift 2
        ;;
      --target=*)
        TARGET_DIR="${1#*=}"
        shift
        ;;
      --agents-mode)
        [ "$#" -ge 2 ] || die "--agents-mode requires a value"
        AGENTS_MODE="$2"
        shift 2
        ;;
      --agents-mode=*)
        AGENTS_MODE="${1#*=}"
        shift
        ;;
      --yes|-y)
        YES=1
        shift
        ;;
      --force)
        FORCE=1
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        die "Unknown option: $1"
        ;;
    esac
  done
}

main() {
  parse_args "$@"

  case "$AGENTS_MODE" in
    ask|merge|preserve|replace|backup) ;;
    *) die "Unsupported --agents-mode: $AGENTS_MODE" ;;
  esac

  mkdir -p "$TARGET_DIR"
  TARGET_DIR="$(cd "$TARGET_DIR" && pwd)"

  [ "$TARGET_DIR" != "$SOURCE_ROOT" ] || die "Refusing to install into the source Harness repository itself."

  info "Source: $SOURCE_ROOT"
  info "Target: $TARGET_DIR"

  install_harness_tree
  install_agents

  cat <<EOF

Harness installed.

Next steps:
  cd "$TARGET_DIR"
  bash .harness/scripts/verify.sh

If AGENTS.md was preserved, review AGENTS.harness.md and merge the parts you want into AGENTS.md.
EOF
}

main "$@"
