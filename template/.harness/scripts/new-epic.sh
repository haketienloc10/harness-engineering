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
  bash .harness/scripts/new-epic.sh "long task name"

Create a new Harness Epic container for a long-running task.
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

write_run_index_if_missing() {
  if [ ! -f "$RUN_INDEX" ]; then
    cat > "$RUN_INDEX" <<'EOF'
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
- BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF
- FAILED_VERIFICATION
- CANCELLED
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

replace_placeholders() {
  local file="$1"

  if command -v perl >/dev/null 2>&1; then
    perl -pi -e "s/<EPIC-ID>/$EPIC_ID/g; s/<TASK-SLUG>/$slug/g; s/<CREATED-AT>/$NOW/g; s/<UPDATED-AT>/$NOW/g" "$file"
  else
    sed -i "s/<EPIC-ID>/$EPIC_ID/g; s/<TASK-SLUG>/$slug/g; s/<CREATED-AT>/$NOW/g; s/<UPDATED-AT>/$NOW/g" "$file"
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

next_epic_sequence() {
  local last

  last="$(
    {
      find "$RUNS_DIR" -maxdepth 1 -type d -name "EPIC-${TODAY}-[0-9][0-9][0-9]-*" -printf '%f\n' 2>/dev/null \
        | sed -E "s/^EPIC-${TODAY}-([0-9][0-9][0-9])-.*$/\\1/"
      if [ -f "$RUN_INDEX" ]; then
        sed -n -E "s/.*EPIC-${TODAY}-([0-9][0-9][0-9])-.*$/\\1/p" "$RUN_INDEX"
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

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi

raw_slug="${*:-untitled}"
slug="$(slugify "$raw_slug")"

[ -d "$TEMPLATES_DIR" ] || die "Templates directory not found: $TEMPLATES_DIR"

TODAY="$(date +%Y%m%d)"
NOW="$(date -Iseconds)"

mkdir -p "$RUNS_DIR"
write_run_index_if_missing
migrate_run_index_if_needed

N="$(next_epic_sequence)"
while true; do
  EPIC_ID="EPIC-${TODAY}-$(printf "%03d" "$N")-${slug}"
  EPIC_DIR="$RUNS_DIR/$EPIC_ID"
  if [ ! -d "$EPIC_DIR" ]; then
    break
  fi
  N=$((N + 1))
done

mkdir -p "$EPIC_DIR/runs"

cp "$TEMPLATES_DIR/epic.yaml.template" "$EPIC_DIR/epic.yaml"
cp "$TEMPLATES_DIR/00-epic-overview.template.md" "$EPIC_DIR/00-epic-overview.md"
cp "$TEMPLATES_DIR/01-epic-roadmap.template.md" "$EPIC_DIR/01-roadmap.md"
cp "$TEMPLATES_DIR/02-epic-acceptance-matrix.template.md" "$EPIC_DIR/02-acceptance-matrix.md"
cp "$TEMPLATES_DIR/03-epic-contract-review.template.md" "$EPIC_DIR/03-epic-contract-review.md"
cp "$TEMPLATES_DIR/04-epic-run-index.template.md" "$EPIC_DIR/04-run-index.md"
cp "$TEMPLATES_DIR/03-epic-decision-log.template.md" "$EPIC_DIR/05-decision-log.md"

for f in "$EPIC_DIR"/*; do
  [ -f "$f" ] && replace_placeholders "$f"
done

append_run_index_row "| epic |  | $EPIC_ID | $slug | active |  |  | agent | $NOW | $NOW |"

echo "Created Harness epic:"
echo ".harness/runs/$EPIC_ID/"
