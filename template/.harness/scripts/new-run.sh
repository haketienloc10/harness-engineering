#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS_DIR="$ROOT_DIR/.harness"
RUNS_DIR="$HARNESS_DIR/runs"
TEMPLATES_DIR="$HARNESS_DIR/templates"
RUN_INDEX="$RUNS_DIR/RUN_INDEX.md"

usage() {
  cat <<'EOF'
Usage:
  bash .harness/scripts/new-run.sh "task name"
  bash .harness/scripts/new-run.sh "task name" --epic EPIC-YYYYMMDD-NNN-task
  bash .harness/scripts/new-run.sh --epic EPIC-YYYYMMDD-NNN-task "task name"

Create a new Harness run. When --epic is provided, link the run to the Epic.
EOF
}

die() {
  echo "Error: $*" >&2
  exit 1
}

slugify() {
  local raw="$1"
  local slug

  slug="$(printf "%s" "$raw" \
    | tr '[:upper:]' '[:lower:]' \
    | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//; s/-+/-/g')"

  if [ -z "$slug" ]; then
    slug="untitled"
  fi

  printf "%s" "$slug"
}

parse_args() {
  TASK_PARTS=()
  EPIC_ID=""

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --epic)
        [ "$#" -ge 2 ] || die "--epic requires an EPIC-ID"
        EPIC_ID="$2"
        shift 2
        ;;
      --epic=*)
        EPIC_ID="${1#*=}"
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        TASK_PARTS+=("$1")
        shift
        ;;
    esac
  done

  raw_slug="${TASK_PARTS[*]:-untitled}"
}

replace_placeholders() {
  local file="$1"

  if command -v perl >/dev/null 2>&1; then
    perl -pi -e "s/<RUN-ID>/$RUN_ID/g; s/<TASK-SLUG>/$slug/g; s/<CREATED-AT>/$NOW/g; s/<UPDATED-AT>/$NOW/g" "$file"
  else
    sed -i "s/<RUN-ID>/$RUN_ID/g; s/<TASK-SLUG>/$slug/g; s/<CREATED-AT>/$NOW/g; s/<UPDATED-AT>/$NOW/g" "$file"
  fi
}

write_run_index_if_missing() {
  if [ ! -f "$RUN_INDEX" ]; then
    cat > "$RUN_INDEX" <<'EOF'
# Harness Run Index

| Run ID | Task | Status | Branch | Worktree | Owner | Started At | Last Updated |
|---|---|---|---|---|---|---|---|
EOF
  fi
}

set_related_epic_notes() {
  local file="$1"

  if [ -z "$EPIC_ID" ]; then
    return
  fi

  if command -v perl >/dev/null 2>&1; then
    perl -0pi -e "s/(## Related Epic\\n\\n)None/\${1}$EPIC_ID/g" "$file"
  else
    sed -i "/^## Related Epic$/,/^$/ s/^None$/$EPIC_ID/" "$file"
  fi
}

parse_args "$@"

slug="$(slugify "$raw_slug")"

if [ -n "$EPIC_ID" ] && [ ! -d "$HARNESS_DIR/epics/$EPIC_ID" ]; then
  die "Epic directory not found: $HARNESS_DIR/epics/$EPIC_ID"
fi

[ -d "$TEMPLATES_DIR" ] || die "Templates directory not found: $TEMPLATES_DIR"

TODAY="$(date +%Y%m%d)"
NOW="$(date -Iseconds)"

mkdir -p "$RUNS_DIR"

N=1
while true; do
  RUN_ID="RUN-${TODAY}-$(printf "%03d" "$N")-${slug}"
  RUN_DIR="$RUNS_DIR/$RUN_ID"
  if [ ! -d "$RUN_DIR" ]; then
    break
  fi
  N=$((N + 1))
done

mkdir -p "$RUN_DIR"

cp "$TEMPLATES_DIR/run.yaml.template" "$RUN_DIR/run.yaml"
cp "$TEMPLATES_DIR/00-input.template.md" "$RUN_DIR/00-input.md"
cp "$TEMPLATES_DIR/01-planner-brief.template.md" "$RUN_DIR/01-planner-brief.md"
cp "$TEMPLATES_DIR/02-implementation-contract.template.md" "$RUN_DIR/02-implementation-contract.md"
cp "$TEMPLATES_DIR/03-evaluator-contract-review.template.md" "$RUN_DIR/03-evaluator-contract-review.md"
cp "$TEMPLATES_DIR/04-generator-worklog.template.md" "$RUN_DIR/04-generator-worklog.md"
cp "$TEMPLATES_DIR/05-evaluator-report.template.md" "$RUN_DIR/05-evaluator-report.md"
cp "$TEMPLATES_DIR/06-fix-report.template.md" "$RUN_DIR/06-fix-report.md"
cp "$TEMPLATES_DIR/07-final-summary.template.md" "$RUN_DIR/07-final-summary.md"

for f in "$RUN_DIR"/*; do
  if [ -f "$f" ]; then
    replace_placeholders "$f"
    set_related_epic_notes "$f"
  fi
done

write_run_index_if_missing

printf "| %s | %s | created |  |  | agent | %s | %s |\n" "$RUN_ID" "$slug" "$NOW" "$NOW" >> "$RUN_INDEX"

echo "Created Harness run:"
echo "$RUN_DIR"

if [ -n "$EPIC_ID" ]; then
  bash "$HARNESS_DIR/scripts/link-run-to-epic.sh" "$EPIC_ID" "$RUN_ID"
fi
