#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS_DIR="$ROOT_DIR/.harness"
RUNS_DIR="$HARNESS_DIR/runs"
RUN_INDEX="$RUNS_DIR/RUN_INDEX.md"

usage() {
  cat <<'EOF'
Usage:
  bash .harness/scripts/link-run-to-epic.sh EPIC-YYYYMMDD-NNN-task RUN-ID

Refresh the child run index for an existing Harness child run.
EOF
}

die() {
  echo "Error: $*" >&2
  exit 1
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

set_nested_active_run_id() {
  local file="$1"
  local value="$2"

  if grep -qE "^[[:space:]]+active_run_id:" "$file"; then
    if command -v perl >/dev/null 2>&1; then
      perl -pi -e "s/^[[:space:]]+active_run_id:.*$/  active_run_id: \"$value\"/" "$file"
    else
      sed -i "s/^[[:space:]]*active_run_id:.*$/  active_run_id: \"$value\"/" "$file"
    fi
  fi
}

ensure_epic_run_index() {
  local file="$1"

  if [ ! -f "$file" ]; then
    cat > "$file" <<'EOF'
# Epic Run Index

| Run ID | Task | Status | Branch | Started At | Updated At | Result |
|---|---|---|---|---|---|---|

## Run Status Values

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
  fi
}

append_or_update_run_index() {
  local file="$1"
  local row="$2"

  if grep -qF "| $RUN_ID |" "$file"; then
    if command -v perl >/dev/null 2>&1; then
      ROW="$row" RUN_ID="$RUN_ID" perl -pi -e 's/^\| \Q$ENV{RUN_ID}\E \|.*$/$ENV{ROW}/' "$file"
    else
      sed -i "/^| $RUN_ID |/c\\$row" "$file"
    fi
  elif grep -qE '^## Run Status Values' "$file"; then
    local tmp
    tmp="${file}.tmp.$$"
    awk -v row="$row" '
      BEGIN { inserted = 0; has_prev = 0; prev = "" }
      /^## Run Status Values/ && inserted == 0 {
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
    ' "$file" > "$tmp"
    mv "$tmp" "$file"
  else
    printf "%s\n" "$row" >> "$file"
  fi
}

update_global_epic_index() {
  local row

  row="| child-run | $EPIC_ID | $RUN_ID | $TASK | $STATUS | $BRANCH |  | agent | $NOW | $NOW |"

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
- BLOCKED_FOR_EXECUTOR_UNAVAILABLE
- FAILED_VERIFICATION
- CANCELLED
EOF
  fi

  if grep -qF "| child-run | $EPIC_ID | $RUN_ID |" "$RUN_INDEX"; then
    if command -v perl >/dev/null 2>&1; then
      ROW="$row" EPIC_ID="$EPIC_ID" RUN_ID="$RUN_ID" perl -pi -e 's/^\| child-run \| \Q$ENV{EPIC_ID}\E \| \Q$ENV{RUN_ID}\E \|.*$/$ENV{ROW}/' "$RUN_INDEX"
    else
      sed -i "/^| child-run | $EPIC_ID | $RUN_ID |/c\\$row" "$RUN_INDEX"
    fi
  elif grep -qE '^## Status Values' "$RUN_INDEX"; then
    local tmp
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

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi

[ "$#" -eq 2 ] || die "Expected EPIC-ID and RUN-ID"

EPIC_ID="$1"
RUN_ID="$2"
if [ -d "$RUNS_DIR/$EPIC_ID" ]; then
  EPIC_DIR="$RUNS_DIR/$EPIC_ID"
else
  die "Epic directory not found: $RUNS_DIR/$EPIC_ID"
fi

if [ -d "$EPIC_DIR/runs/$RUN_ID" ]; then
  RUN_DIR="$EPIC_DIR/runs/$RUN_ID"
else
  die "Child run directory not found: $EPIC_DIR/runs/$RUN_ID"
fi
RUN_YAML="$RUN_DIR/run.yaml"
EPIC_YAML="$EPIC_DIR/epic.yaml"
EPIC_RUN_INDEX="$EPIC_DIR/04-run-index.md"
NOW="$(date -Iseconds)"

[ -d "$EPIC_DIR" ] || die "Epic directory not found: $EPIC_DIR"
[ -d "$RUN_DIR" ] || die "Run directory not found: $RUN_DIR"
[ -f "$RUN_YAML" ] || die "Run YAML not found: $RUN_YAML"

TASK="$(sed -n 's/^task:[[:space:]]*//p' "$RUN_YAML" | head -n 1 | sed -E 's/^"//; s/"$//')"
STATUS="$(sed -n 's/^state:[[:space:]]*//p' "$RUN_YAML" | head -n 1)"
BRANCH="$(sed -n 's/^branch:[[:space:]]*//p' "$RUN_YAML" | head -n 1 | tr -d '"')"

TASK="${TASK:-}"
STATUS="${STATUS:-CREATED}"
BRANCH="${BRANCH:-}"

set_yaml_field "$RUN_YAML" "parent_epic" "$EPIC_ID"

if [ -f "$EPIC_YAML" ]; then
  set_nested_active_run_id "$EPIC_YAML" "$RUN_ID"
  set_yaml_field "$EPIC_YAML" "updated_at" "$NOW"
fi

ensure_epic_run_index "$EPIC_RUN_INDEX"
append_or_update_run_index "$EPIC_RUN_INDEX" "| $RUN_ID | $TASK | $STATUS | $BRANCH | $NOW | $NOW | linked |"
update_global_epic_index

echo "Linked run to epic:"
echo "  Epic: $EPIC_ID"
echo "  Run:  $RUN_ID"
