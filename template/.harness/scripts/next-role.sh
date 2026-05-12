#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS_DIR="$ROOT_DIR/.harness"
TEMPLATES_DIR="$HARNESS_DIR/templates"

usage() {
  cat <<'EOF'
Usage:
  bash .harness/scripts/next-role.sh .harness/runs/RUN-...
  bash .harness/scripts/next-role.sh .harness/runs/EPIC-.../runs/RUN-...

Determine the next Harness lifecycle role and required artifact from run.yaml.
EOF
}

die() {
  echo "Error: $*" >&2
  exit 1
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

artifact_to_role() {
  case "$1" in
    01-planner-brief.md|02-implementation-contract.md)
      printf "planner"
      ;;
    03-evaluator-contract-review.md)
      printf "contract_reviewer"
      ;;
    04-generator-worklog.md|06-fix-report.md)
      printf "generator"
      ;;
    05-evaluator-report.md)
      printf "evaluator"
      ;;
    *)
      printf "none"
      ;;
  esac
}

state_next_role() {
  case "$1" in
    CREATED|PLANNING|CONTRACTING|REJECTED_FOR_REPLAN)
      printf "planner"
      ;;
    CONTRACT_REVIEW)
      printf "contract_reviewer"
      ;;
    APPROVED_FOR_IMPLEMENTATION|GENERATING|FAILED_VERIFICATION)
      printf "generator"
      ;;
    EVALUATING)
      printf "evaluator"
      ;;
    BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF)
      artifact_to_role "$NEXT_REQUIRED_ARTIFACT"
      ;;
    COMPLETED|CANCELLED)
      printf "none"
      ;;
  esac
}

state_required_artifact() {
  case "$1" in
    CREATED|PLANNING)
      printf "01-planner-brief.md"
      ;;
    CONTRACTING|REJECTED_FOR_REPLAN)
      printf "02-implementation-contract.md"
      ;;
    CONTRACT_REVIEW)
      printf "03-evaluator-contract-review.md"
      ;;
    APPROVED_FOR_IMPLEMENTATION|GENERATING)
      printf "04-generator-worklog.md"
      ;;
    EVALUATING)
      printf "05-evaluator-report.md"
      ;;
    FAILED_VERIFICATION)
      printf "06-fix-report.md"
      ;;
    BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF)
      printf "%s" "${NEXT_REQUIRED_ARTIFACT:-HANDOFF.md}"
      ;;
    COMPLETED|CANCELLED)
      printf "none"
      ;;
  esac
}

required_files_for_role() {
  local role="$1"

  case "$role" in
    planner)
      printf -- "- %s/run.yaml\n- %s/00-input.md\n- .harness/guides/LIFECYCLE_ORCHESTRATION.md\n" "$RUN_DIR" "$RUN_DIR"
      ;;
    contract_reviewer)
      printf -- "- %s/run.yaml\n- %s/01-planner-brief.md\n- %s/02-implementation-contract.md\n- .harness/guides/LIFECYCLE_ORCHESTRATION.md\n" "$RUN_DIR" "$RUN_DIR" "$RUN_DIR"
      ;;
    generator)
      printf -- "- %s/run.yaml\n- %s/02-implementation-contract.md\n- %s/03-evaluator-contract-review.md\n- .harness/guides/LIFECYCLE_ORCHESTRATION.md\n" "$RUN_DIR" "$RUN_DIR" "$RUN_DIR"
      ;;
    evaluator)
      printf -- "- %s/run.yaml\n- %s/02-implementation-contract.md\n- %s/03-evaluator-contract-review.md\n- %s/04-generator-worklog.md\n- .harness/guides/LIFECYCLE_ORCHESTRATION.md\n" "$RUN_DIR" "$RUN_DIR" "$RUN_DIR" "$RUN_DIR"
      ;;
    *)
      printf -- "- %s/run.yaml\n" "$RUN_DIR"
      ;;
  esac
}

forbidden_actions_for_role() {
  case "$1" in
    planner)
      printf -- "- Do not implement application code.\n- Do not approve your own contract.\n- Do not claim evaluator evidence.\n"
      ;;
    contract_reviewer)
      printf -- "- Do not implement application code.\n- Do not rewrite the contract silently.\n- Do not approve unless criteria are measurable and executable.\n"
      ;;
    generator)
      printf -- "- Do not start unless the contract is approved.\n- Do not change scope beyond the approved contract.\n- Do not evaluate your own output.\n"
      ;;
    evaluator)
      printf -- "- Do not patch implementation to make tests pass.\n- Do not rely on hidden reasoning or memory.\n- Do not approve without command/runtime evidence.\n"
      ;;
    *)
      printf -- "- Do not skip lifecycle states.\n"
      ;;
  esac
}

decision_criteria_for_role() {
  case "$1" in
    planner)
      printf -- "- Plan is bounded.\n- Contract has measurable acceptance criteria.\n- Verification path is executable.\n"
      ;;
    contract_reviewer)
      printf -- "- Approve only if Generator can implement safely from visible artifacts.\n- Reject if scope is oversized, vague, or unverifiable.\n"
      ;;
    generator)
      printf -- "- Implement only approved scope.\n- Record changed files, commands, and notable decisions.\n"
      ;;
    evaluator)
      printf -- "- Verify every required behaviour with real evidence.\n- Include command output or runtime evidence.\n- Pass only if evidence supports the contract.\n"
      ;;
    *)
      printf -- "- Preserve run.yaml as source of truth.\n"
      ;;
  esac
}

write_handoff() {
  local template="$TEMPLATES_DIR/HANDOFF.template.md"
  local handoff="$RUN_DIR/HANDOFF.md"
  local required_files forbidden_actions decision_criteria parent completed_role

  required_files="$(required_files_for_role "$NEXT_ROLE")"
  forbidden_actions="$(forbidden_actions_for_role "$NEXT_ROLE")"
  decision_criteria="$(decision_criteria_for_role "$NEXT_ROLE")"
  parent="${PARENT_EPIC:-none}"
  [ "$parent" = "null" ] && parent="none"
  completed_role="${CURRENT_ROLE:-orchestrator}"

  if [ -f "$template" ]; then
    cp "$template" "$handoff"
  else
    : > "$handoff"
  fi

  RUN_DIR_VALUE="$RUN_DIR" \
    PARENT_EPIC_VALUE="$parent" \
    STATE_VALUE="$STATE" \
    COMPLETED_ROLE_VALUE="$completed_role" \
    NEXT_ROLE_VALUE="$NEXT_ROLE" \
    REQUIRED_FILES_VALUE="$required_files" \
    FORBIDDEN_ACTIONS_VALUE="$forbidden_actions" \
    REQUIRED_ARTIFACT_VALUE="$REQUIRED_ARTIFACT" \
    DECISION_CRITERIA_VALUE="$decision_criteria" \
    perl -0pi -e 's/<RUN-DIR>/$ENV{RUN_DIR_VALUE}/g; s/<PARENT-EPIC>/$ENV{PARENT_EPIC_VALUE}/g; s/<STATE>/$ENV{STATE_VALUE}/g; s/<COMPLETED-ROLE>/$ENV{COMPLETED_ROLE_VALUE}/g; s/<NEXT-ROLE>/$ENV{NEXT_ROLE_VALUE}/g; s/<REQUIRED-FILES>/$ENV{REQUIRED_FILES_VALUE}/g; s/<FORBIDDEN-ACTIONS>/$ENV{FORBIDDEN_ACTIONS_VALUE}/g; s/<REQUIRED-ARTIFACT>/$ENV{REQUIRED_ARTIFACT_VALUE}/g; s/<DECISION-CRITERIA>/$ENV{DECISION_CRITERIA_VALUE}/g' "$handoff"

  printf "%s" "$handoff"
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
[ -f "$RUN_YAML" ] || die "run.yaml not found: $RUN_YAML"

STATE="$(yaml_get state)"
CURRENT_ROLE="$(yaml_get current_role)"
PARENT_EPIC="$(yaml_get parent_epic)"
NEXT_REQUIRED_ARTIFACT="$(yaml_get next_required_artifact)"
APPROVED_FOR_IMPLEMENTATION="$(yaml_get approved_for_implementation)"
GENERATOR_ALLOWED="$(yaml_get generator_allowed)"

[ -n "$STATE" ] || die "state is empty in run.yaml"
recognized_state "$STATE" || die "Unrecognized lifecycle state: $STATE"

if [ "$STATE" = "APPROVED_FOR_IMPLEMENTATION" ] || [ "$STATE" = "GENERATING" ]; then
  [ "$APPROVED_FOR_IMPLEMENTATION" = "true" ] || die "$STATE requires approved_for_implementation: true"
  [ "$GENERATOR_ALLOWED" = "true" ] || die "$STATE requires generator_allowed: true"
fi

NEXT_ROLE="$(state_next_role "$STATE")"
REQUIRED_ARTIFACT="$(state_required_artifact "$STATE")"
HANDOFF="false"

if [ "$NEXT_ROLE" != "none" ]; then
  EXECUTOR_VALUE="$(role_executor_get "$NEXT_ROLE")"
  if [ "$STATE" = "BLOCKED_FOR_INDEPENDENT_ROLE_HANDOFF" ] || [ "$EXECUTOR_VALUE" = "unavailable" ] || [ "$EXECUTOR_VALUE" = "blocked" ]; then
    HANDOFF="$(write_handoff)"
  fi
fi

printf "RUN_DIR=%s\n" "$RUN_DIR"
printf "STATE=%s\n" "$STATE"
printf "NEXT_ROLE=%s\n" "$NEXT_ROLE"
printf "REQUIRED_ARTIFACT=%s\n" "$REQUIRED_ARTIFACT"
printf "HANDOFF=%s\n" "$HANDOFF"
