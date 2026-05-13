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
  role_executor_field_get "$role" "executor_type"
}

role_executor_field_get() {
  local role="$1"
  local field="$2"
  awk -v role="$role" -v field="$field" '
    /^role_executors:/ { in_roles = 1; next }
    in_roles == 1 && /^[^[:space:]]/ { in_roles = 0 }
    in_roles == 1 {
      pattern = "^[[:space:]]+" role ":[[:space:]]*"
      if ($0 ~ pattern) {
        in_role = 1
        next
      }
      if (in_role == 1 && $0 ~ /^[[:space:]]+[a-zA-Z_]+:[[:space:]]*$/) {
        in_role = 0
      }
      if (in_role == 1) {
        field_pattern = "^[[:space:]]+" field ":[[:space:]]*"
        if ($0 ~ field_pattern) {
          sub(field_pattern, "", $0)
          gsub(/^"|"$/, "", $0)
          print $0
          exit
        }
      }
    }
  ' "$RUN_YAML"
}

require_role_spawned() {
  local role="$1"
  local executor_type
  local executor_id

  executor_type="$(role_executor_field_get "$role" executor_type)"
  executor_id="$(role_executor_field_get "$role" executor_id)"

  [ "$executor_type" = "subagent" ] || die "$role must use executor_type: subagent"
  [ -n "$executor_id" ] || die "$role requires a spawned subagent executor_id"
}

recognized_state() {
  case "$1" in
    CREATED|PLANNING|CONTRACTING|CONTRACT_REVIEW|APPROVED_FOR_IMPLEMENTATION|GENERATING|EVALUATING|COMPLETED|REJECTED_FOR_REPLAN|BLOCKED_FOR_EXECUTOR_UNAVAILABLE|FAILED_VERIFICATION|CANCELLED)
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

manifest_field_get() {
  local field="$1"
  sed -n -E "s/^- ${field}:[[:space:]]*//p" "$RUN_DIR/run-manifest.md" | head -n 1 | sed -E 's/^"//; s/"$//'
}

artifact_metadata_field_get() {
  local file="$1"
  local field="$2"

  awk -v field="$field" '
    /^```yaml[[:space:]]*$/ {
      in_yaml = 1
      next
    }
    in_yaml == 1 && /^```[[:space:]]*$/ {
      exit
    }
    in_yaml == 1 {
      pattern = "^" field ":[[:space:]]*"
      if ($0 ~ pattern) {
        sub(pattern, "", $0)
        gsub(/^"|"$/, "", $0)
        print $0
        exit
      }
    }
  ' "$RUN_DIR/$file"
}

require_artifact_metadata_field() {
  local file="$1"
  local field="$2"
  local value

  value="$(artifact_metadata_field_get "$file" "$field")"
  [ -n "$value" ] || die "$file missing runtime metadata field: $field"
  [ "$value" != "<required>" ] || die "$file has unresolved runtime metadata field: $field"
}

require_role_artifact_metadata() {
  local file="$1"
  local role="$2"
  local template_source="$3"
  local executor_type
  local executor_id
  local actual_template_source

  require_artifact_metadata_field "$file" role
  require_artifact_metadata_field "$file" executor_type
  require_artifact_metadata_field "$file" executor_id
  require_artifact_metadata_field "$file" template_source
  require_artifact_metadata_field "$file" started_at
  require_artifact_metadata_field "$file" completed_at

  executor_type="$(artifact_metadata_field_get "$file" executor_type)"
  executor_id="$(artifact_metadata_field_get "$file" executor_id)"
  actual_template_source="$(artifact_metadata_field_get "$file" template_source)"

  [ "$(artifact_metadata_field_get "$file" role)" = "$role" ] || die "$file must set role: $role"
  [ "$executor_type" = "subagent" ] || die "$file must set executor_type: subagent"
  [ -n "$executor_id" ] || die "$file requires executor_id"
  [ "$actual_template_source" = "$template_source" ] || die "$file must set template_source: $template_source"
}

require_contract_review_approved() {
  require_completed_artifact "03-contract-review.md"
  grep -qE '^- Status:[[:space:]]*approved[[:space:]]*$' "$RUN_DIR/03-contract-review.md" || die "generator_allowed: true requires 03-contract-review.md Status: approved"
}

require_manifest() {
  require_file "run-manifest.md"
  grep -qF -- "- mode: template_subagents_required" "$RUN_DIR/run-manifest.md" || die "run-manifest.md must set mode: template_subagents_required"
  grep -qF -- "- fallback_allowed: false" "$RUN_DIR/run-manifest.md" || die "run-manifest.md must set fallback_allowed: false"
  grep -qF -- "- coordinator_source_edits_allowed: true" "$RUN_DIR/run-manifest.md" && die "Coordinator source edits are forbidden"
  grep -qF -- "- coordinator_role_work_allowed: true" "$RUN_DIR/run-manifest.md" && die "Coordinator role work is forbidden"
  grep -qF "planner_template: .harness/subagents/planner.md" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing planner template source"
  grep -qF "contract_reviewer_template: .harness/subagents/contract-reviewer.md" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing contract reviewer template source"
  grep -qF "generator_template: .harness/subagents/generator.md" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing generator template source"
  grep -qF "evaluator_template: .harness/subagents/evaluator.md" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing evaluator template source"

  if [ "$STATE" != "CREATED" ] && [ "$STATE" != "BLOCKED_FOR_EXECUTOR_UNAVAILABLE" ] && [ "$STATE" != "CANCELLED" ]; then
    case "$(manifest_field_get subagent_runtime_available)" in
      true)
        ;;
      unknown|false|"")
        die "run-manifest.md cannot have subagent_runtime_available: unknown/false after CREATED"
        ;;
      *)
        die "run-manifest.md has invalid subagent_runtime_available value"
        ;;
    esac
  fi
}

require_completed_artifact() {
  local file="$1"

  require_file "$file"

  grep -q '<required>' "$RUN_DIR/$file" && die "Required artifact still contains <required>: $file"
  grep -q '<command>' "$RUN_DIR/$file" && die "Required artifact still contains <command>: $file"
  grep -qE '^\.\.\.$' "$RUN_DIR/$file" && die "Required artifact still contains placeholder content: $file"
  grep -qE 'Status: APPROVED \| REJECTED|Status: PASS \| FAIL' "$RUN_DIR/$file" && die "Required artifact still contains unresolved status choice: $file"
  grep -qE 'Pass/Fail|Yes/No|Continue / Sequence / Worktree / Block|Low/Medium/High|Manual/E2E/API/Unit/Build' "$RUN_DIR/$file" && die "Required artifact still contains unresolved option placeholder: $file"
}

require_completed_role_artifact() {
  local file="$1"
  local role="$2"
  local template_source="$3"

  require_completed_artifact "$file"
  require_role_artifact_metadata "$file" "$role" "$template_source"
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
      require_completed_role_artifact "01-planner-brief.md" "Planner" ".harness/subagents/planner.md"
      require_role_spawned "planner"
      ;;
    CONTRACT_REVIEW)
      require_file "00-input.md"
      require_completed_role_artifact "01-planner-brief.md" "Planner" ".harness/subagents/planner.md"
      require_completed_role_artifact "02-implementation-contract.md" "Planner" ".harness/subagents/planner.md"
      require_role_spawned "planner"
      ;;
    APPROVED_FOR_IMPLEMENTATION|GENERATING)
      require_file "00-input.md"
      require_completed_role_artifact "01-planner-brief.md" "Planner" ".harness/subagents/planner.md"
      require_completed_role_artifact "02-implementation-contract.md" "Planner" ".harness/subagents/planner.md"
      require_completed_role_artifact "03-contract-review.md" "ContractReviewer" ".harness/subagents/contract-reviewer.md"
      require_role_spawned "planner"
      require_role_spawned "contract_reviewer"
      ;;
    EVALUATING)
      require_file "00-input.md"
      require_completed_role_artifact "01-planner-brief.md" "Planner" ".harness/subagents/planner.md"
      require_completed_role_artifact "02-implementation-contract.md" "Planner" ".harness/subagents/planner.md"
      require_completed_role_artifact "03-contract-review.md" "ContractReviewer" ".harness/subagents/contract-reviewer.md"
      require_completed_role_artifact "04-implementation-report.md" "Generator" ".harness/subagents/generator.md"
      require_role_spawned "planner"
      require_role_spawned "contract_reviewer"
      require_role_spawned "generator"
      ;;
    COMPLETED)
      require_file "00-input.md"
      require_completed_role_artifact "01-planner-brief.md" "Planner" ".harness/subagents/planner.md"
      require_completed_role_artifact "02-implementation-contract.md" "Planner" ".harness/subagents/planner.md"
      require_completed_role_artifact "03-contract-review.md" "ContractReviewer" ".harness/subagents/contract-reviewer.md"
      require_completed_role_artifact "04-implementation-report.md" "Generator" ".harness/subagents/generator.md"
      require_completed_role_artifact "05-evaluator-report.md" "Evaluator" ".harness/subagents/evaluator.md"
      require_completed_artifact "07-final-summary.md"
      require_role_spawned "planner"
      require_role_spawned "contract_reviewer"
      require_role_spawned "generator"
      require_role_spawned "evaluator"
      ;;
    REJECTED_FOR_REPLAN)
      require_file "00-input.md"
      require_completed_role_artifact "01-planner-brief.md" "Planner" ".harness/subagents/planner.md"
      require_completed_role_artifact "02-implementation-contract.md" "Planner" ".harness/subagents/planner.md"
      ;;
    BLOCKED_FOR_EXECUTOR_UNAVAILABLE)
      [ -n "$(yaml_get blocked_reason)" ] || die "BLOCKED_FOR_EXECUTOR_UNAVAILABLE requires blocked_reason"
      grep -qF -- "- run_status: blocked" "$RUN_DIR/run-manifest.md" || die "Blocked run requires run-manifest.md run_status: blocked"
      grep -qF -- "- subagent_runtime_available: false" "$RUN_DIR/run-manifest.md" || die "Blocked run requires subagent_runtime_available: false"
      ;;
    FAILED_VERIFICATION)
      require_completed_role_artifact "05-evaluator-report.md" "Evaluator" ".harness/subagents/evaluator.md"
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
  local generator_report_executor_id
  local evaluator_report_generator_executor_id
  local evaluator_report_executor_id
  local same_executor_as_generator

  [ -f "$report" ] || die "COMPLETED requires 05-evaluator-report.md"
  grep -qE '^## Commands Executed' "$report" || die "Evaluator report must include a Commands Executed section"
  grep -qE '^## Evidence' "$report" || die "Evaluator report must include an Evidence section"
  grep -qE '^- Status:[[:space:]]*pass[[:space:]]*$' "$report" || die "COMPLETED requires 05-evaluator-report.md Decision Status: pass"

  if grep -q '<command>' "$report"; then
    die "Evaluator report still contains placeholder command evidence"
  fi

  if awk '/^## Evidence/ { in_evidence = 1; next } /^## / && in_evidence == 1 { in_evidence = 0 } in_evidence == 1 { print }' "$report" | grep -q '^\.\.\.$'; then
    die "Evaluator report Evidence section still contains placeholder content"
  fi

  generator_report_executor_id="$(artifact_metadata_field_get "04-implementation-report.md" executor_id)"
  evaluator_report_generator_executor_id="$(artifact_metadata_field_get "05-evaluator-report.md" generator_executor_id)"
  evaluator_report_executor_id="$(artifact_metadata_field_get "05-evaluator-report.md" evaluator_executor_id)"
  same_executor_as_generator="$(artifact_metadata_field_get "05-evaluator-report.md" same_executor_as_generator)"

  [ -n "$evaluator_report_generator_executor_id" ] || die "05-evaluator-report.md missing generator_executor_id"
  [ -n "$evaluator_report_executor_id" ] || die "05-evaluator-report.md missing evaluator_executor_id"
  [ "$same_executor_as_generator" = "false" ] || die "05-evaluator-report.md must set same_executor_as_generator: false"
  [ "$evaluator_report_generator_executor_id" = "$generator_report_executor_id" ] || die "05-evaluator-report.md generator_executor_id must match 04-implementation-report.md executor_id"
  [ "$evaluator_report_executor_id" = "$(artifact_metadata_field_get "05-evaluator-report.md" executor_id)" ] || die "05-evaluator-report.md evaluator_executor_id must match executor_id"
  [ "$evaluator_report_executor_id" != "$evaluator_report_generator_executor_id" ] || die "Evaluator cannot be the same executor_id as Generator"
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
GENERATOR_EXECUTOR_ID="$(role_executor_field_get generator executor_id)"
EVALUATOR_EXECUTOR_ID="$(role_executor_field_get evaluator executor_id)"

[ -n "$STATE" ] || die "state is empty in run.yaml"
recognized_state "$STATE" || die "Unrecognized lifecycle state: $STATE"

validate_path_shape
require_manifest
require_artifacts_for_state

if [ "$STATE" = "GENERATING" ]; then
  [ "$APPROVED_FOR_IMPLEMENTATION" = "true" ] || die "GENERATING is invalid unless approved_for_implementation: true"
  [ "$GENERATOR_ALLOWED" = "true" ] || die "GENERATING is invalid unless generator_allowed: true"
fi

if [ "$GENERATOR_ALLOWED" = "true" ]; then
  require_contract_review_approved
fi

if [ "$STATE" = "COMPLETED" ]; then
  validate_evaluator_report
fi

if [ -n "$GENERATOR_EXECUTOR_ID" ] && [ -n "$EVALUATOR_EXECUTOR_ID" ] && [ "$GENERATOR_EXECUTOR_ID" = "$EVALUATOR_EXECUTOR_ID" ]; then
  die "Evaluator cannot be the same executor_id as Generator"
fi

if grep -qE 'fallback_single_session|task_tool|external_agent_session|isolated_process' "$RUN_YAML" "$RUN_DIR/run-manifest.md"; then
  die "Run contains invalid fallback or non-subagent executor language"
fi

if [ "$STATE" = "APPROVED_FOR_IMPLEMENTATION" ] && [ "$APPROVED_FOR_IMPLEMENTATION" != "true" ]; then
  warn "APPROVED_FOR_IMPLEMENTATION should set approved_for_implementation: true before Generator runs"
fi

echo "OK: $RUN_DIR"
