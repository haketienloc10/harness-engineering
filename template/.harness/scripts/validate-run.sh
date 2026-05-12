#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNS_DIR="$ROOT_DIR/.harness/runs"

usage() {
  cat <<'EOF'
Usage:
  bash .harness/scripts/validate-run.sh .harness/runs/RUN-...
  bash .harness/scripts/validate-run.sh .harness/runs/EPIC-.../runs/RUN-...

Validate Harness lifecycle state and required run artifacts.
EOF
}

die() {
  echo "ERROR: $*" >&2
  exit 1
}

warn() {
  echo "WARN: $*" >&2
}

yaml_get() {
  local key="$1"
  sed -n -E "s/^${key}:[[:space:]]*//p" "$RUN_YAML" | head -n 1 | sed -E 's/^"//; s/"$//'
}

role_executor_get() {
  local role="$1"
  awk -v role="$role" '
    /^role_executors:/ { in_roles = 1; next }
    in_roles == 1 && /^[^[:space:]]/ { in_roles = 0 }
    in_roles == 1 {
      pattern = "^[[:space:]]+" role ":[[:space:]]*"
      if ($0 ~ pattern) {
        sub(pattern, "", $0)
        gsub(/^"|"$/, "", $0)
        print $0
        exit
      }
    }
  ' "$RUN_YAML"
}

recognized_state() {
  case "$1" in
    CREATED|PLANNING|CONTRACTING|CONTRACT_REVIEW|APPROVED_FOR_IMPLEMENTATION|GENERATING|EVALUATING|COMPLETED|REJECTED_FOR_REPLAN|BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF|FAILED_VERIFICATION|CANCELLED)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

require_file() {
  local file="$1"
  [ -f "$RUN_DIR/$file" ] || die "Required artifact missing for state $STATE: $file"
}

require_artifacts_for_state() {
  case "$STATE" in
    CREATED)
      require_file "00-input.md"
      ;;
    PLANNING)
      require_file "00-input.md"
      ;;
    CONTRACTING)
      require_file "00-input.md"
      require_file "01-planner-brief.md"
      ;;
    CONTRACT_REVIEW)
      require_file "00-input.md"
      require_file "01-planner-brief.md"
      require_file "02-implementation-contract.md"
      ;;
    APPROVED_FOR_IMPLEMENTATION|GENERATING)
      require_file "00-input.md"
      require_file "01-planner-brief.md"
      require_file "02-implementation-contract.md"
      require_file "03-evaluator-contract-review.md"
      ;;
    EVALUATING)
      require_file "00-input.md"
      require_file "01-planner-brief.md"
      require_file "02-implementation-contract.md"
      require_file "03-evaluator-contract-review.md"
      require_file "04-generator-worklog.md"
      ;;
    COMPLETED)
      require_file "00-input.md"
      require_file "01-planner-brief.md"
      require_file "02-implementation-contract.md"
      require_file "03-evaluator-contract-review.md"
      require_file "04-generator-worklog.md"
      require_file "05-evaluator-report.md"
      require_file "07-final-summary.md"
      ;;
    REJECTED_FOR_REPLAN)
      require_file "00-input.md"
      require_file "01-planner-brief.md"
      require_file "02-implementation-contract.md"
      ;;
    BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF)
      require_file "HANDOFF.md"
      ;;
    FAILED_VERIFICATION)
      require_file "05-evaluator-report.md"
      ;;
    CANCELLED)
      require_file "00-input.md"
      ;;
  esac
}

validate_path_shape() {
  local base parent
  base="$(basename "$RUN_DIR")"
  parent="$(yaml_get parent_epic)"

  case "$base" in
    RUN-[0-9][0-9][0-9]-*)
      case "$RUN_DIR" in
        "$RUNS_DIR"/EPIC-*/runs/RUN-*)
          ;;
        *)
          die "Child run path must be under .harness/runs/EPIC-*/runs/RUN-*"
          ;;
      esac
      ;;
  esac

  if [ -n "$parent" ] && [ "$parent" != "null" ]; then
    case "$RUN_DIR" in
      "$RUNS_DIR/$parent"/runs/RUN-*)
        ;;
      *)
        die "parent_epic is $parent but run path is not under .harness/runs/$parent/runs/RUN-*"
        ;;
    esac
  fi
}

validate_evaluator_report() {
  local report="$RUN_DIR/05-evaluator-report.md"

  [ -f "$report" ] || die "COMPLETED requires 05-evaluator-report.md"
  grep -qE '^## Commands Executed' "$report" || die "Evaluator report must include a Commands Executed section"
  grep -qE '^## Evidence' "$report" || die "Evaluator report must include an Evidence section"

  if grep -q '<command>' "$report"; then
    die "Evaluator report still contains placeholder command evidence"
  fi

  if awk '/^## Evidence/ { in_evidence = 1; next } /^## / && in_evidence == 1 { in_evidence = 0 } in_evidence == 1 { print }' "$report" | grep -q '^\.\.\.$'; then
    die "Evaluator report Evidence section still contains placeholder content"
  fi
}

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi

[ "$#" -eq 1 ] || die "Expected run directory path"

RUN_DIR_INPUT="$1"
[ -d "$RUN_DIR_INPUT" ] || die "Run directory not found: $RUN_DIR_INPUT"
RUN_DIR="$(cd "$RUN_DIR_INPUT" && pwd -P)"
RUN_YAML="$RUN_DIR/run.yaml"

[ -f "$RUN_YAML" ] || die "run.yaml does not exist: $RUN_YAML"

STATE="$(yaml_get state)"
APPROVED_FOR_IMPLEMENTATION="$(yaml_get approved_for_implementation)"
GENERATOR_ALLOWED="$(yaml_get generator_allowed)"
GENERATOR_EXECUTOR="$(role_executor_get generator)"
EVALUATOR_EXECUTOR="$(role_executor_get evaluator)"

[ -n "$STATE" ] || die "state is empty in run.yaml"
recognized_state "$STATE" || die "Unrecognized lifecycle state: $STATE"

validate_path_shape
require_artifacts_for_state

if [ "$STATE" = "GENERATING" ]; then
  [ "$APPROVED_FOR_IMPLEMENTATION" = "true" ] || die "GENERATING is invalid unless approved_for_implementation: true"
  [ "$GENERATOR_ALLOWED" = "true" ] || die "GENERATING is invalid unless generator_allowed: true"
fi

if [ "$STATE" = "COMPLETED" ]; then
  validate_evaluator_report
fi

if [ -n "$GENERATOR_EXECUTOR" ] && [ -n "$EVALUATOR_EXECUTOR" ] && [ "$GENERATOR_EXECUTOR" = "$EVALUATOR_EXECUTOR" ]; then
  die "Evaluator cannot be the same executor as Generator"
fi

if [ "$STATE" = "APPROVED_FOR_IMPLEMENTATION" ] && [ "$APPROVED_FOR_IMPLEMENTATION" != "true" ]; then
  warn "APPROVED_FOR_IMPLEMENTATION should set approved_for_implementation: true before Generator runs"
fi

echo "OK: $RUN_DIR"
