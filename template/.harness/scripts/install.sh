#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_HARNESS_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SOURCE_TEMPLATE_DIR="$(cd "$SOURCE_HARNESS_DIR/.." && pwd)"
SOURCE_REPO_DIR="$(cd "$SOURCE_TEMPLATE_DIR/.." && pwd)"

TARGET_DIR="$(pwd)"
AGENTS_MODE="ask"
YES=0
FORCE=0
DRY_RUN=0

usage() {
  cat <<'EOF'
Usage:
  bash template/.harness/scripts/install.sh [options]

Install the Harness seed template into a target repository.

Options:
  --target PATH              Repository to install into. Defaults to current directory.
  --agents-mode MODE         ask|merge|preserve|replace|backup
  --dry-run                  Print planned operations without writing files.
  --yes, -y                  Do not prompt. If AGENTS.md exists and mode is omitted, uses merge.
  --force                    Allow ownership-safe update of an existing .harness/.
  --help, -h                 Show this help.

Ownership-safe rules:
  - Do not overwrite .harness/project/* if it already exists.
  - Do not overwrite .harness/codebase/* if it already exists.
  - Do not reset .harness/runs/RUN_INDEX.md if it already exists.
  - Do not copy run history from the template.
  - Do not delete legacy .harness/epics/ if it already exists.
  - Do not overwrite .harness/backlog/HARNESS_BACKLOG.md if it already exists.
  - Kernel folders may be replaced on update:
    .harness/guides/
    .harness/subagents/
    .harness/workflows/
    .harness/templates/
    .harness/project-templates/
    .harness/scripts/
EOF
}

die() {
  echo "Error: $*" >&2
  exit 1
}

info() {
  echo "==> $*"
}

planned() {
  if [ "$DRY_RUN" -eq 1 ]; then
    echo "[dry-run] $*"
  fi
}

run_mkdir() {
  if [ "$DRY_RUN" -eq 1 ]; then
    planned "mkdir -p $1"
  else
    mkdir -p "$1"
  fi
}

run_cp() {
  local source="$1"
  local dest="$2"

  if [ "$DRY_RUN" -eq 1 ]; then
    planned "copy $source -> $dest"
  else
    mkdir -p "$(dirname "$dest")"
    cp "$source" "$dest"
  fi
}

run_write_file() {
  local file="$1"

  if [ "$DRY_RUN" -eq 1 ]; then
    planned "write $file"
    cat >/dev/null
  else
    mkdir -p "$(dirname "$file")"
    cat > "$file"
  fi
}

backup_file() {
  local file="$1"
  local stamp backup

  stamp="$(date +%Y%m%d%H%M%S)"
  backup="${file}.backup-${stamp}"
  if [ -e "$backup" ]; then
    backup="${backup}-$$"
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    planned "backup $file -> $backup"
  else
    cp "$file" "$backup"
  fi
  printf "%s" "$backup"
}

copy_dir_replace() {
  local source="$1"
  local dest="$2"

  [ -d "$source" ] || die "Source directory not found: $source"

  if [ "$DRY_RUN" -eq 1 ]; then
    planned "replace directory $dest from $source"
  else
    rm -rf "$dest"
    mkdir -p "$(dirname "$dest")"
    cp -R "$source" "$dest"
  fi
}

write_clean_run_index() {
  local dest="$1"

  run_write_file "$dest" <<'EOF'
# Harness Run Index

| Type | Parent | ID | Task | Status | Branch | Worktree | Owner | Started At | Last Updated |
|---|---|---|---|---|---|---|---|---|---|

## Status Values

- active
- CREATED
- PLANNING
- CONTRACTING
- CONTRACT_REVIEW
- APPROVED_FOR_IMPLEMENTATION
- GENERATING
- EVALUATING
- COMPLETED
- REJECTED_FOR_REPLAN
- BLOCKED_FOR_EXECUTOR_UNAVAILABLE
- FAILED_VERIFICATION
- CANCELLED
EOF
}

write_installation_note() {
  local dest="$1"
  local installed_at

  installed_at="$(date -Iseconds)"
  run_write_file "$dest" <<EOF
# Harness Installation

Installed at: $installed_at

This target repository owns its installed \`.harness/\` tree.

## Ownership Rules

- \`.harness/project/*\` belongs to this target repository. The installer creates missing files only and does not overwrite existing project adapter files.
- \`.harness/codebase/*\` belongs to this target repository. The installer creates missing files only and does not overwrite existing source-navigation or change-impact files.
- \`.harness/runs/RUN_INDEX.md\`, root runs, Epic containers, and child runs belong to this target repository. The installer does not reset existing run history.
- Legacy \`.harness/epics/*\`, if present from older Harness installs, belongs to this target repository and is never deleted by the installer.
- \`.harness/backlog/HARNESS_BACKLOG.md\` belongs to this target repository. The installer does not overwrite it if it already exists.
- Kernel folders may be replaced during an explicit update:
  - \`.harness/guides/\`
  - \`.harness/subagents/\`
  - \`.harness/workflows/\`
  - \`.harness/templates/\`
  - \`.harness/project-templates/\`
  - \`.harness/scripts/\`
- Seeded Harness workflow skill files are copied into \`.harness/skills/\` without deleting other local skill files.

## Recommended Next Steps

Ask your agent:

\`\`\`txt
Read \`.harness/HARNESS_SKILLS.md\` and run the \`project-sync\` Harness workflow skill.
Then run \`codebase-sync\` if source-navigation or change-impact docs are missing or stale.
\`\`\`

No native-agent skill installation is required.
EOF
}

write_merged_agents() {
  local source_agents="$1"
  local existing_agents="$2"
  local dest_agents="$3"

  if [ "$DRY_RUN" -eq 1 ]; then
    planned "merge $source_agents + $existing_agents -> $dest_agents"
    return
  fi

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

  printf "Select [1-4] (default: 1): "
  read -r choice

  case "${choice:-1}" in
    1|merge) AGENTS_MODE="merge" ;;
    2|preserve) AGENTS_MODE="preserve" ;;
    3|replace) AGENTS_MODE="replace" ;;
    4|backup) AGENTS_MODE="backup" ;;
    *) die "Invalid AGENTS.md choice: $choice" ;;
  esac
}

install_agents() {
  local source_agents="$SOURCE_TEMPLATE_DIR/AGENTS.md"
  local target_agents="$TARGET_DIR/AGENTS.md"
  local harness_agents="$TARGET_DIR/AGENTS.harness.md"
  local backup

  [ -f "$source_agents" ] || die "Source AGENTS.md not found: $source_agents"

  if [ ! -e "$target_agents" ]; then
    run_cp "$source_agents" "$target_agents"
    info "Installed AGENTS.md"
    return
  fi

  if [ "$AGENTS_MODE" = "ask" ] && [ "$YES" -eq 1 ]; then
    AGENTS_MODE="merge"
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
      run_cp "$source_agents" "$harness_agents"
      info "Preserved AGENTS.md and wrote Harness instructions to AGENTS.harness.md"
      ;;
    replace|backup)
      backup="$(backup_file "$target_agents")"
      run_cp "$source_agents" "$target_agents"
      info "Installed Harness AGENTS.md and backed up the original to $backup"
      ;;
    *)
      die "Unsupported --agents-mode: $AGENTS_MODE"
      ;;
  esac
}

install_project_adapters() {
  local source_dir="$SOURCE_HARNESS_DIR/project-templates"
  local target_project="$TARGET_DIR/.harness/project"
  local template_file base dest

  run_mkdir "$target_project"

  for template_file in "$source_dir"/*.template.md; do
    [ -f "$template_file" ] || continue
    base="$(basename "$template_file")"
    dest="$target_project/${base%.template.md}.md"
    if [ -e "$dest" ]; then
      info "Preserved existing project adapter: $dest"
    else
      run_cp "$template_file" "$dest"
      info "Created project adapter: $dest"
    fi
  done
}

install_codebase_docs() {
  local source_dir="$SOURCE_HARNESS_DIR/codebase"
  local target_codebase="$TARGET_DIR/.harness/codebase"
  local source_file base dest

  run_mkdir "$target_codebase"

  for source_file in "$source_dir"/*.md; do
    [ -f "$source_file" ] || continue
    base="$(basename "$source_file")"
    dest="$target_codebase/$base"
    if [ -e "$dest" ]; then
      info "Preserved existing codebase doc: $dest"
    else
      run_cp "$source_file" "$dest"
      info "Created codebase doc: $dest"
    fi
  done
}

install_harness_tree() {
  local target_harness="$TARGET_DIR/.harness"

  if [ -d "$target_harness" ] && [ "$FORCE" -ne 1 ] && [ "$YES" -ne 1 ]; then
    printf ".harness/ already exists in target. Update kernel folders? [y/N]: "
    local answer
    read -r answer
    case "$answer" in
      y|Y|yes|YES) ;;
      *) die "Cancelled because .harness/ already exists. Re-run with --force to update." ;;
    esac
  fi

  run_mkdir "$target_harness"
  run_cp "$SOURCE_HARNESS_DIR/README.md" "$target_harness/README.md"
  run_cp "$SOURCE_HARNESS_DIR/HARNESS_SKILLS.md" "$target_harness/HARNESS_SKILLS.md"
  write_installation_note "$target_harness/INSTALLATION.md"

  copy_dir_replace "$SOURCE_HARNESS_DIR/guides" "$target_harness/guides"
  copy_dir_replace "$SOURCE_HARNESS_DIR/subagents" "$target_harness/subagents"
  copy_dir_replace "$SOURCE_HARNESS_DIR/workflows" "$target_harness/workflows"
  copy_dir_replace "$SOURCE_HARNESS_DIR/templates" "$target_harness/templates"
  copy_dir_replace "$SOURCE_HARNESS_DIR/project-templates" "$target_harness/project-templates"
  copy_dir_replace "$SOURCE_HARNESS_DIR/scripts" "$target_harness/scripts"
  run_mkdir "$target_harness/skills"
  run_cp "$SOURCE_HARNESS_DIR/skills/project-sync.md" "$target_harness/skills/project-sync.md"
  run_cp "$SOURCE_HARNESS_DIR/skills/codebase-sync.md" "$target_harness/skills/codebase-sync.md"

  if [ "$DRY_RUN" -eq 0 ]; then
    chmod +x "$target_harness/scripts"/*.sh 2>/dev/null || true
  fi

  install_project_adapters
  install_codebase_docs

  run_mkdir "$target_harness/backlog"
  if [ -f "$target_harness/backlog/HARNESS_BACKLOG.md" ]; then
    info "Preserved existing backlog: $target_harness/backlog/HARNESS_BACKLOG.md"
  else
    run_cp "$SOURCE_HARNESS_DIR/backlog/HARNESS_BACKLOG.md" "$target_harness/backlog/HARNESS_BACKLOG.md"
    info "Created backlog: $target_harness/backlog/HARNESS_BACKLOG.md"
  fi

  run_mkdir "$target_harness/runs"
  if [ -f "$target_harness/runs/RUN_INDEX.md" ]; then
    info "Preserved existing run index: $target_harness/runs/RUN_INDEX.md"
  else
    write_clean_run_index "$target_harness/runs/RUN_INDEX.md"
    info "Created run index: $target_harness/runs/RUN_INDEX.md"
  fi

  if [ "$DRY_RUN" -eq 0 ]; then
    touch "$target_harness/runs/.gitkeep"
  else
    planned "touch $target_harness/runs/.gitkeep"
  fi

  if [ -d "$target_harness/epics" ]; then
    info "Preserved legacy epic directory: $target_harness/epics"
  fi

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
      --dry-run)
        DRY_RUN=1
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

  [ -d "$SOURCE_HARNESS_DIR/guides" ] || die "Invalid template: missing $SOURCE_HARNESS_DIR/guides"
  [ -d "$SOURCE_HARNESS_DIR/subagents" ] || die "Invalid template: missing $SOURCE_HARNESS_DIR/subagents"
  [ -d "$SOURCE_HARNESS_DIR/workflows" ] || die "Invalid template: missing $SOURCE_HARNESS_DIR/workflows"
  [ -d "$SOURCE_HARNESS_DIR/codebase" ] || die "Invalid template: missing $SOURCE_HARNESS_DIR/codebase"
  [ -f "$SOURCE_TEMPLATE_DIR/AGENTS.md" ] || die "Invalid template: missing $SOURCE_TEMPLATE_DIR/AGENTS.md"

  if [ "$DRY_RUN" -eq 0 ]; then
    mkdir -p "$TARGET_DIR"
    TARGET_DIR="$(cd "$TARGET_DIR" && pwd)"
  fi

  if [ "$DRY_RUN" -eq 0 ]; then
    local target_real source_repo_real source_template_real
    target_real="$(cd "$TARGET_DIR" && pwd)"
    source_repo_real="$(cd "$SOURCE_REPO_DIR" && pwd)"
    source_template_real="$(cd "$SOURCE_TEMPLATE_DIR" && pwd)"
    [ "$target_real" != "$source_repo_real" ] || die "Refusing to install into the seed repository itself."
    [ "$target_real" != "$source_template_real" ] || die "Refusing to install into the template source directory itself."
  fi

  info "Source template: $SOURCE_TEMPLATE_DIR"
  info "Target: $TARGET_DIR"

  install_harness_tree
  install_agents

  cat <<EOF

Harness installed.

Next steps:
  cd "$TARGET_DIR"
  Ask your agent:
    Read .harness/HARNESS_SKILLS.md and run the project-sync Harness workflow skill.
    Then run codebase-sync if source-navigation or change-impact docs are missing or stale.
  No native-agent skill installation is required.

If AGENTS.md was preserved, review AGENTS.harness.md and merge the parts you want into AGENTS.md.
EOF
}

main "$@"
