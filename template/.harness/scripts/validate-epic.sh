#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNS_DIR="$ROOT_DIR/.harness/runs"

usage() {
  cat <<'EOF'
Usage:
  bash .harness/scripts/validate-epic.sh .harness/runs/EPIC-...

Validate Harness Epic structure and closure readiness.
EOF
}

die() {
  echo "ERROR: $*" >&2
  exit 1
}

yaml_get() {
  local key="$1"
  sed -n -E "s/^${key}:[[:space:]]*//p" "$EPIC_YAML" | head -n 1 | sed -E 's/^"//; s/"$//'
}

decomposition_get() {
  local key="$1"
  awk -v key="$key" '
    /^decomposition:/ { in_section = 1; next }
    in_section == 1 && /^[^[:space:]]/ { in_section = 0 }
    in_section == 1 {
      pattern = "^[[:space:]]+" key ":[[:space:]]*"
      if ($0 ~ pattern) {
        sub(pattern, "", $0)
        gsub(/^"|"$/, "", $0)
        print $0
        exit
      }
    }
  ' "$EPIC_YAML"
}

require_file() {
  local file="$1"
  [ -f "$EPIC_DIR/$file" ] || die "Required Epic artifact missing: $file"
}

child_run_count() {
  find "$EPIC_DIR/runs" -mindepth 1 -maxdepth 1 -type d -name 'RUN-*' 2>/dev/null | wc -l | tr -d '[:space:]'
}

validate_acceptance_matrix_for_closure() {
  awk -F'|' '
    BEGIN { failures = 0 }
    /^\| AC-[0-9]+/ {
      id = trim($2)
      required = trim($4)
      covered = trim($5)
      evidence = trim($6)
      status = tolower(trim($7))
      if (required == "Yes") {
        if (covered == "" || evidence == "" || status != "pass") {
          printf "ERROR: Required acceptance row is not closure-ready: %s\n", id > "/dev/stderr"
          failures = 1
        }
      }
    }
    END { exit failures }
    function trim(value) {
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
      return value
    }
  ' "$EPIC_DIR/02-acceptance-matrix.md" || die "Required Epic acceptance criteria must have coverage, evidence, and Pass status before closure"
}

validate_child_run_evidence() {
  local run_id report run_ids

  run_ids="$(grep -oE 'RUN-[0-9]{3}-[A-Za-z0-9._-]+|RUN-[0-9]{8}-[0-9]{3}-[A-Za-z0-9._-]+' "$EPIC_DIR/02-acceptance-matrix.md" | sort -u || true)"
  [ -n "$run_ids" ] || die "Epic acceptance evidence must reference child run IDs"

  printf "%s\n" "$run_ids" | while IFS= read -r run_id; do
    [ -n "$run_id" ] || continue
    report="$EPIC_DIR/runs/$run_id/05-evaluator-report.md"
    [ -f "$report" ] || die "Acceptance matrix references $run_id but evaluator report is missing"
  done
}

validate_completion_checklist() {
  if grep -qE '^- \[ \]' "$EPIC_DIR/02-acceptance-matrix.md"; then
    die "Epic completion checklist still has unchecked items"
  fi
}

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi

[ "$#" -eq 1 ] || die "Expected Epic directory path"

EPIC_DIR_INPUT="$1"
[ -d "$EPIC_DIR_INPUT" ] || die "Epic directory not found: $EPIC_DIR_INPUT"
EPIC_DIR="$(cd "$EPIC_DIR_INPUT" && pwd -P)"
EPIC_YAML="$EPIC_DIR/epic.yaml"

[ -f "$EPIC_YAML" ] || die "epic.yaml does not exist: $EPIC_YAML"

case "$EPIC_DIR" in
  "$RUNS_DIR"/EPIC-*)
    ;;
  *)
    die "Epic path must be under .harness/runs/EPIC-*"
    ;;
esac

require_file "00-epic-overview.md"
require_file "01-roadmap.md"
require_file "02-acceptance-matrix.md"
require_file "03-epic-contract-review.md"
require_file "04-run-index.md"
require_file "05-decision-log.md"
[ -d "$EPIC_DIR/runs" ] || die "Epic runs directory missing: runs/"

STATUS="$(yaml_get status)"
MIN_CHILD_RUNS="$(decomposition_get minimum_required_child_runs)"
[ -n "$MIN_CHILD_RUNS" ] || MIN_CHILD_RUNS="2"

case "$STATUS" in
  completed|closed|done|COMPLETED|CLOSED|DONE)
    COUNT="$(child_run_count)"
    [ "$COUNT" -ge "$MIN_CHILD_RUNS" ] || die "Epic closure requires at least $MIN_CHILD_RUNS child runs; found $COUNT"
    validate_acceptance_matrix_for_closure
    validate_child_run_evidence
    validate_completion_checklist
    ;;
esac

echo "OK: $EPIC_DIR"
