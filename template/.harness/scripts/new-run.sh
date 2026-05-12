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
  bash .harness/scripts/new-run.sh --within EPIC-YYYYMMDD-NNN-task "task name"
  bash .harness/scripts/new-run.sh "task name" --within EPIC-YYYYMMDD-NNN-task
  bash .harness/scripts/new-run.sh --epic EPIC-YYYYMMDD-NNN-task "task name"
  bash .harness/scripts/new-run.sh --force-normal-run "bounded task name"

Create a new Harness run. Use --within for a child run inside an Epic container.
--epic remains as a backward-compatible alias for --within.
Oversized task signals are blocked by default. --force-normal-run is explicit
degraded override and is not production-grade for broad or multi-phase work.
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
  FORCE_NORMAL_RUN=0

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --force-normal-run)
        FORCE_NORMAL_RUN=1
        shift
        ;;
      --within)
        [ "$#" -ge 2 ] || die "--within requires an EPIC-ID"
        EPIC_ID="$2"
        shift 2
        ;;
      --within=*)
        EPIC_ID="${1#*=}"
        shift
        ;;
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

oversized_signal_reason() {
  local s="$1"
  local match

  match="$(printf "%s\n" "$s" | sed -n -E 's/.*(^|-)phase-([0-9]+-[0-9]+)($|-).*/phase-\2/p' | head -n 1)"
  if [ -n "$match" ]; then
    printf "contains multi-phase signal: %s" "$match"
    return 0
  fi

  match="$(printf "%s\n" "$s" | sed -n -E 's/.*(^|-)part-([0-9]+-[0-9]+)($|-).*/part-\2/p' | head -n 1)"
  if [ -n "$match" ]; then
    printf "contains multi-part signal: %s" "$match"
    return 0
  fi

  if printf "%s\n" "$s" | grep -Eq '(^|-)phase($|-)'; then
    printf "contains multi-phase signal: phase"
    return 0
  fi

  if printf "%s\n" "$s" | grep -Eq '(^|-)part($|-)'; then
    printf "contains multi-part signal: part"
    return 0
  fi

  if printf "%s\n" "$s" | grep -Eq '(^|-)core-loop($|-)'; then
    printf "contains broad workflow signal: core-loop"
    return 0
  fi

  if printf "%s\n" "$s" | grep -Eq '(^|-)end-to-end($|-)'; then
    printf "contains broad verification signal: end-to-end"
    return 0
  fi

  for match in full complete mvp large long multi workflow module milestone; do
    if printf "%s\n" "$s" | grep -Eq "(^|-)$match($|-)"; then
      printf "contains oversized task signal: %s" "$match"
      return 0
    fi
  done

  return 1
}

enforce_run_classification() {
  local reason
  local epic_slug

  if reason="$(oversized_signal_reason "$slug")"; then
    if [ "$FORCE_NORMAL_RUN" -eq 1 ]; then
      echo "WARNING: Task appears too large for a normal run." >&2
      echo "Reason: $reason" >&2
      echo "Continuing only because --force-normal-run was provided." >&2
      echo "This is not production-grade for broad, multi-phase, Epic, or child-run work." >&2
      return
    fi

    echo "ERROR: Task appears too large for a normal run." >&2
    echo "Reason: $reason" >&2
    if [ -n "$EPIC_ID" ]; then
      echo "This child run appears too broad. Create smaller independently verifiable child runs under: $EPIC_ID" >&2
    else
      epic_slug="$(printf "%s\n" "$slug" | sed -E 's/(^|-)phase-[0-9]+-[0-9]+($|-)/-/g; s/(^|-)part-[0-9]+-[0-9]+($|-)/-/g; s/(^|-)phase($|-)/-/g; s/(^|-)part($|-)/-/g; s/^-+//; s/-+$//; s/-+/-/g')"
      [ -n "$epic_slug" ] || epic_slug="$slug"
      echo "Use: bash .harness/scripts/new-epic.sh \"${epic_slug}\"" >&2
    fi
    echo "Override only for an explicitly bounded, non-production exception with: --force-normal-run" >&2
    exit 1
  fi
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

| Type | Parent | ID | Task | Status | Branch | Worktree | Owner | Started At | Last Updated |
|---|---|---|---|---|---|---|---|---|---|

## Status Values

- active
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
  fi
}

migrate_run_index_if_needed() {
  local tmp

  [ -f "$RUN_INDEX" ] || return
  grep -qF "| Type | Parent | ID |" "$RUN_INDEX" && return

  tmp="${RUN_INDEX}.tmp.$$"
  awk '
    BEGIN { converted_header = 0; skip_separator = 0 }
    /^\| Run ID \| Task \| Status \| Branch \| Worktree \| Owner \| Started At \| Last Updated \|$/ {
      print "| Type | Parent | ID | Task | Status | Branch | Worktree | Owner | Started At | Last Updated |"
      converted_header = 1
      skip_separator = 1
      next
    }
    skip_separator == 1 && /^\|---\|---\|---\|---\|---\|---\|---\|---\|$/ {
      print "|---|---|---|---|---|---|---|---|---|---|"
      skip_separator = 0
      next
    }
    converted_header == 1 && /^\| RUN-/ {
      sub(/^\| /, "| run |  | ")
      print
      next
    }
    { print }
  ' "$RUN_INDEX" > "$tmp"
  mv "$tmp" "$RUN_INDEX"
}

set_yaml_field() {
  local file="$1"
  local key="$2"
  local value="$3"

  if grep -qE "^${key}:" "$file"; then
    if command -v perl >/dev/null 2>&1; then
      perl -pi -e "s/^${key}:.*$/${key}: $value/" "$file"
    else
      sed -i "s/^${key}:.*$/${key}: $value/" "$file"
    fi
  else
    printf "%s: %s\n" "$key" "$value" >> "$file"
  fi
}

append_run_index_row() {
  local row="$1"
  local tmp

  if grep -qE '^## Status Values' "$RUN_INDEX"; then
    tmp="${RUN_INDEX}.tmp.$$"
    awk -v row="$row" '
      BEGIN { inserted = 0; has_prev = 0; prev = "" }
      /^## Status Values/ && inserted == 0 {
        if (has_prev == 1 && prev != "") {
          print prev
        }
        print row
        print ""
        print
        inserted = 1
        has_prev = 0
        next
      }
      {
        if (has_prev == 1) {
          print prev
        }
        prev = $0
        has_prev = 1
      }
      END {
        if (has_prev == 1) {
          print prev
        }
        if (inserted == 0) {
          print row
        }
      }
    ' "$RUN_INDEX" > "$tmp"
    mv "$tmp" "$RUN_INDEX"
  else
    printf "%s\n" "$row" >> "$RUN_INDEX"
  fi
}

next_root_run_sequence() {
  local last

  last="$(
    {
      find "$RUNS_DIR" -maxdepth 1 -type d -name "RUN-${TODAY}-[0-9][0-9][0-9]-*" -printf '%f\n' 2>/dev/null \
        | sed -E "s/^RUN-${TODAY}-([0-9][0-9][0-9])-.*$/\\1/"
      if [ -f "$RUN_INDEX" ]; then
        sed -n -E "s/.*RUN-${TODAY}-([0-9][0-9][0-9])-.*$/\\1/p" "$RUN_INDEX"
      fi
    } \
    | sort -n \
    | tail -n 1
  )"

  if [ -n "$last" ]; then
    printf "%d" "$((10#$last + 1))"
  else
    printf "1"
  fi
}

next_child_run_sequence() {
  local epic_runs_dir="$1"
  local last

  last="$(find "$epic_runs_dir" -maxdepth 1 -type d -name 'RUN-[0-9][0-9][0-9]-*' -printf '%f\n' 2>/dev/null \
    | sed -E 's/^RUN-([0-9][0-9][0-9])-.*$/\1/' \
    | sort -n \
    | tail -n 1)"

  if [ -n "$last" ]; then
    printf "%d" "$((10#$last + 1))"
  else
    printf "1"
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

[ -d "$TEMPLATES_DIR" ] || die "Templates directory not found: $TEMPLATES_DIR"

enforce_run_classification

TODAY="$(date +%Y%m%d)"
NOW="$(date -Iseconds)"

mkdir -p "$RUNS_DIR"
write_run_index_if_missing
migrate_run_index_if_needed

if [ -n "$EPIC_ID" ]; then
  EPIC_DIR="$RUNS_DIR/$EPIC_ID"
  [ -d "$EPIC_DIR" ] || die "Epic container not found: $EPIC_DIR"
  [ -f "$EPIC_DIR/epic.yaml" ] || die "Epic YAML not found: $EPIC_DIR/epic.yaml"
  mkdir -p "$EPIC_DIR/runs"

  N="$(next_child_run_sequence "$EPIC_DIR/runs")"
  while true; do
    RUN_ID="RUN-$(printf "%03d" "$N")-${slug}"
    RUN_DIR="$EPIC_DIR/runs/$RUN_ID"
    if [ ! -d "$RUN_DIR" ]; then
      break
    fi
    N=$((N + 1))
  done
else
  N="$(next_root_run_sequence)"
  while true; do
    RUN_ID="RUN-${TODAY}-$(printf "%03d" "$N")-${slug}"
    RUN_DIR="$RUNS_DIR/$RUN_ID"
    if [ ! -d "$RUN_DIR" ]; then
      break
    fi
    N=$((N + 1))
  done
fi

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

if [ -n "$EPIC_ID" ]; then
  set_yaml_field "$RUN_DIR/run.yaml" "epic_id" "$EPIC_ID"
  set_yaml_field "$RUN_DIR/run.yaml" "classification" "epic_child_run"
  append_run_index_row "| child-run | $EPIC_ID | $RUN_ID | $slug | created |  |  | agent | $NOW | $NOW |"
else
  append_run_index_row "| run |  | $RUN_ID | $slug | created |  |  | agent | $NOW | $NOW |"
fi


echo "Created Harness run:"
echo "$RUN_DIR"

if [ -n "$EPIC_ID" ]; then
  bash "$HARNESS_DIR/scripts/link-run-to-epic.sh" "$EPIC_ID" "$RUN_ID"
fi
