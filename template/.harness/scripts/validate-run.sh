#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNS_DIR="$ROOT_DIR/.harness/runs"
CODEX_AGENTS_DIR="$ROOT_DIR/.codex/agents"

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

runtime_get() {
  local key="$1"
  awk -v key="$key" '
    /^runtime:/ { in_runtime = 1; next }
    in_runtime == 1 && /^[^[:space:]]/ { in_runtime = 0 }
    in_runtime == 1 {
      pattern = "^[[:space:]]+" key ":[[:space:]]*"
      if ($0 ~ pattern) {
        sub(pattern, "", $0)
        gsub(/^"|"$/, "", $0)
        print $0
        exit
      }
    }
  ' "$RUN_YAML"
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
      if (in_role == 1 && $0 ~ /^  [a-zA-Z_]+:[[:space:]]*$/) {
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
  local codex_agent_name="$2"
  local codex_agent_file="$3"
  local executor_type
  local executor_id
  local actual_codex_agent_name
  local actual_codex_agent_file
  local status

  executor_type="$(role_executor_field_get "$role" executor_type)"
  executor_id="$(role_executor_field_get "$role" executor_id)"
  actual_codex_agent_name="$(role_executor_field_get "$role" codex_agent_name)"
  actual_codex_agent_file="$(role_executor_field_get "$role" codex_agent_file)"
  status="$(role_executor_field_get "$role" status)"

  [ "$executor_type" = "subagent" ] || die "$role must use executor_type: subagent"
  [ -n "$executor_id" ] || die "$role requires a spawned subagent executor_id"
  [ "$actual_codex_agent_name" = "$codex_agent_name" ] || die "$role must use codex_agent_name: $codex_agent_name"
  [ "$actual_codex_agent_file" = "$codex_agent_file" ] || die "$role must use codex_agent_file: $codex_agent_file"
  [ "$status" = "completed" ] || die "$role must set status: completed"
  [ "$executor_id" != "coordinator" ] || die "Coordinator cannot be executor_id for $role"
}

recognized_state() {
  case "$1" in
    CREATED|PLANNING|CONTRACT_REVIEW|APPROVED_FOR_IMPLEMENTATION|GENERATING|EVALUATING|COMPLETED|REJECTED_FOR_REPLAN|BLOCKED_FOR_EXECUTOR_UNAVAILABLE|FAILED_VERIFICATION|CANCELLED)
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
    NR == 1 && /^---[[:space:]]*$/ {
      in_frontmatter = 1
      next
    }
    in_frontmatter == 1 && /^---[[:space:]]*$/ {
      exit
    }
    in_frontmatter == 1 {
      pattern = "^" field ":[[:space:]]*"
      if ($0 ~ pattern) {
        sub(pattern, "", $0)
        gsub(/^"|"$/, "", $0)
        print $0
        exit
      }
    }
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
  local codex_agent_name="$3"
  local codex_agent_file="$4"
  local executor_type
  local executor_id
  local actual_codex_agent_name
  local actual_codex_agent_file

  require_artifact_metadata_field "$file" role
  require_artifact_metadata_field "$file" executor_type
  require_artifact_metadata_field "$file" executor_id
  require_artifact_metadata_field "$file" codex_agent_name
  require_artifact_metadata_field "$file" codex_agent_file
  require_artifact_metadata_field "$file" status

  executor_type="$(artifact_metadata_field_get "$file" executor_type)"
  executor_id="$(artifact_metadata_field_get "$file" executor_id)"
  actual_codex_agent_name="$(artifact_metadata_field_get "$file" codex_agent_name)"
  actual_codex_agent_file="$(artifact_metadata_field_get "$file" codex_agent_file)"

  [ "$(artifact_metadata_field_get "$file" role)" = "$role" ] || die "$file must set role: $role"
  [ "$executor_type" = "subagent" ] || die "$file must set executor_type: subagent"
  [ -n "$executor_id" ] || die "$file requires executor_id"
  [ "$executor_id" != "coordinator" ] || die "$file cannot use coordinator as lifecycle role executor_id"
  [ "$actual_codex_agent_name" = "$codex_agent_name" ] || die "$file must set codex_agent_name: $codex_agent_name"
  [ "$actual_codex_agent_file" = "$codex_agent_file" ] || die "$file must set codex_agent_file: $codex_agent_file"
  [ "$(artifact_metadata_field_get "$file" status)" = "completed" ] || die "$file must set status: completed"
}

require_contract_review_approved() {
  require_completed_artifact "03-contract-review.md"
  require_exact_final_status "03-contract-review.md" "approved rejected_requires_revision"
  tail -n 1 "$RUN_DIR/03-contract-review.md" | grep -qE '^- Status:[[:space:]]*approved[[:space:]]*$' || die "generator_allowed: true requires 03-contract-review.md final Status: approved"
}

require_manifest() {
  require_file "run-manifest.md"
  grep -qF -- "- mode: codex_project_subagents_required" "$RUN_DIR/run-manifest.md" || die "run-manifest.md must set mode: codex_project_subagents_required"
  grep -qF -- "- dispatch_mode: codex_project_scoped" "$RUN_DIR/run-manifest.md" || die "run-manifest.md must set dispatch_mode: codex_project_scoped"
  grep -qF -- "- fallback_allowed: false" "$RUN_DIR/run-manifest.md" || die "run-manifest.md must set fallback_allowed: false"
  grep -qF -- "- coordinator_source_edits_allowed: true" "$RUN_DIR/run-manifest.md" && die "Coordinator source edits are forbidden"
  grep -qF -- "- coordinator_role_work_allowed: true" "$RUN_DIR/run-manifest.md" && die "Coordinator role work is forbidden"
  grep -qF "planner_agent_name: harness_planner" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing planner Codex agent name"
  grep -qF "planner_agent_file: .codex/agents/harness-planner.toml" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing planner Codex agent file"
  grep -qF "contract_reviewer_agent_name: harness_contract_reviewer" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing contract reviewer Codex agent name"
  grep -qF "contract_reviewer_agent_file: .codex/agents/harness-contract-reviewer.toml" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing contract reviewer Codex agent file"
  grep -qF "generator_agent_name: harness_generator" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing generator Codex agent name"
  grep -qF "generator_agent_file: .codex/agents/harness-generator.toml" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing generator Codex agent file"
  grep -qF "evaluator_agent_name: harness_evaluator" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing evaluator Codex agent name"
  grep -qF "evaluator_agent_file: .codex/agents/harness-evaluator.toml" "$RUN_DIR/run-manifest.md" || die "run-manifest.md missing evaluator Codex agent file"

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

require_runtime_schema() {
  local runtime_available
  local manifest_runtime_available

  [ "$(runtime_get dispatch_mode)" = "codex_project_scoped" ] || die "runtime.dispatch_mode must be codex_project_scoped"
  [ "$(runtime_get fallback_allowed)" = "false" ] || die "runtime.fallback_allowed must be false"

  runtime_available="$(runtime_get subagent_runtime_available)"
  manifest_runtime_available="$(manifest_field_get subagent_runtime_available)"

  case "$runtime_available" in
    true|false|unknown)
      ;;
    *)
      die "runtime.subagent_runtime_available must be true, false, or unknown"
      ;;
  esac

  [ "$manifest_runtime_available" = "$runtime_available" ] || die "run.yaml and run-manifest.md disagree on subagent_runtime_available"

  if [ "$STATE" != "CREATED" ] && [ "$STATE" != "BLOCKED_FOR_EXECUTOR_UNAVAILABLE" ] && [ "$STATE" != "CANCELLED" ]; then
    [ "$runtime_available" = "true" ] || die "runtime.subagent_runtime_available must be true after lifecycle role execution begins"
  elif [ "$STATE" = "BLOCKED_FOR_EXECUTOR_UNAVAILABLE" ]; then
    [ "$runtime_available" = "false" ] || die "BLOCKED_FOR_EXECUTOR_UNAVAILABLE requires runtime.subagent_runtime_available: false"
  fi
}

require_role_executor_templates() {
  [ -f "$CODEX_AGENTS_DIR/harness-planner.toml" ] || die "missing .codex/agents/harness-planner.toml"
  [ -f "$CODEX_AGENTS_DIR/harness-contract-reviewer.toml" ] || die "missing .codex/agents/harness-contract-reviewer.toml"
  [ -f "$CODEX_AGENTS_DIR/harness-generator.toml" ] || die "missing .codex/agents/harness-generator.toml"
  [ -f "$CODEX_AGENTS_DIR/harness-evaluator.toml" ] || die "missing .codex/agents/harness-evaluator.toml"
  [ "$(role_executor_field_get planner codex_agent_name)" = "harness_planner" ] || die "planner codex_agent_name mismatch"
  [ "$(role_executor_field_get planner codex_agent_file)" = ".codex/agents/harness-planner.toml" ] || die "planner codex_agent_file mismatch"
  [ "$(role_executor_field_get contract_reviewer codex_agent_name)" = "harness_contract_reviewer" ] || die "contract_reviewer codex_agent_name mismatch"
  [ "$(role_executor_field_get contract_reviewer codex_agent_file)" = ".codex/agents/harness-contract-reviewer.toml" ] || die "contract_reviewer codex_agent_file mismatch"
  [ "$(role_executor_field_get generator codex_agent_name)" = "harness_generator" ] || die "generator codex_agent_name mismatch"
  [ "$(role_executor_field_get generator codex_agent_file)" = ".codex/agents/harness-generator.toml" ] || die "generator codex_agent_file mismatch"
  [ "$(role_executor_field_get evaluator codex_agent_name)" = "harness_evaluator" ] || die "evaluator codex_agent_name mismatch"
  [ "$(role_executor_field_get evaluator codex_agent_file)" = ".codex/agents/harness-evaluator.toml" ] || die "evaluator codex_agent_file mismatch"
}

require_no_legacy_artifacts_or_text() {
  [ ! -f "$RUN_DIR/HANDOFF.md" ] || die "Legacy HANDOFF.md is forbidden"
  [ ! -f "$RUN_DIR/03-evaluator-contract-review.md" ] || die "Legacy 03-evaluator-contract-review.md is forbidden"
  [ ! -f "$RUN_DIR/04-generator-worklog.md" ] || die "Legacy 04-generator-worklog.md is forbidden"
  [ ! -f "$RUN_DIR/06-fix-report.md" ] || die "Legacy 06-fix-report.md is forbidden"
  [ ! -f "$RUN_DIR/07-final-summary.md" ] || die "Legacy 07-final-summary.md is forbidden"

  if grep -RniE 'runtime_mode:[[:space:]]*production_multi_session|fallback explicitly allowed|single-session degraded|03-evaluator-contract-review|04-generator-worklog|06-fix-report|07-final-summary' "$RUN_DIR" >/dev/null 2>&1; then
    die "Run contains forbidden legacy workflow text"
  fi
}

require_exact_final_status() {
  local file="$1"
  local allowed_statuses="$2"
  local final_line
  local status
  local matched=0

  final_line="$(tail -n 1 "$RUN_DIR/$file")"
  status="$(printf "%s\n" "$final_line" | sed -n -E 's/^- Status:[[:space:]]*([^[:space:]]+)[[:space:]]*$/\1/p')"
  [ -n "$status" ] || die "$file must end with exactly one '- Status: ...' line"

  for allowed in $allowed_statuses; do
    if [ "$status" = "$allowed" ]; then
      matched=1
    fi
  done

  [ "$matched" -eq 1 ] || die "$file has invalid final status: $status"

  if [ "$(grep -cE '^- Status:' "$RUN_DIR/$file")" -ne 1 ]; then
    die "$file must contain exactly one '- Status:' line"
  fi
}

require_completed_artifact() {
  local file="$1"

  require_file "$file"

  if grep -q '<required>' "$RUN_DIR/$file"; then
    die "Required artifact still contains <required>: $file"
  fi
  if grep -q '<command>' "$RUN_DIR/$file"; then
    die "Required artifact still contains <command>: $file"
  fi
  if grep -qE '^\.\.\.$' "$RUN_DIR/$file"; then
    die "Required artifact still contains placeholder content: $file"
  fi
  if grep -qE 'Status: APPROVED \| REJECTED|Status: PASS \| FAIL' "$RUN_DIR/$file"; then
    die "Required artifact still contains unresolved status choice: $file"
  fi
  if grep -qE 'Pass/Fail|Yes/No|Continue / Sequence / Worktree / Block|Low/Medium/High|Manual/E2E/API/Unit/Build' "$RUN_DIR/$file"; then
    die "Required artifact still contains unresolved option placeholder: $file"
  fi
}

require_completed_role_artifact() {
  local file="$1"
  local role="$2"
  local codex_agent_name="$3"
  local codex_agent_file="$4"

  require_completed_artifact "$file"
  require_role_artifact_metadata "$file" "$role" "$codex_agent_name" "$codex_agent_file"

  case "$file" in
    03-contract-review.md)
      require_exact_final_status "$file" "approved rejected_requires_revision"
      ;;
    05-evaluator-report.md)
      require_exact_final_status "$file" "pass fail blocked_insufficient_evidence"
      ;;
  esac
}

require_artifacts_for_state() {
  case "$STATE" in
    CREATED)
      require_file "00-input.md"
      require_file "01-planner-brief.md"
      require_file "02-implementation-contract.md"
      require_file "03-contract-review.md"
      require_file "04-implementation-report.md"
      require_file "05-evaluator-report.md"
      require_file "06-final-summary.md"
      ;;
    PLANNING)
      require_file "00-input.md"
      ;;
    CONTRACT_REVIEW)
      require_file "00-input.md"
      require_completed_role_artifact "01-planner-brief.md" "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_completed_role_artifact "02-implementation-contract.md" "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_role_spawned "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      ;;
    APPROVED_FOR_IMPLEMENTATION|GENERATING)
      require_file "00-input.md"
      require_completed_role_artifact "01-planner-brief.md" "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_completed_role_artifact "02-implementation-contract.md" "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_completed_role_artifact "03-contract-review.md" "contract_reviewer" "harness_contract_reviewer" ".codex/agents/harness-contract-reviewer.toml"
      require_role_spawned "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_role_spawned "contract_reviewer" "harness_contract_reviewer" ".codex/agents/harness-contract-reviewer.toml"
      ;;
    EVALUATING)
      require_file "00-input.md"
      require_completed_role_artifact "01-planner-brief.md" "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_completed_role_artifact "02-implementation-contract.md" "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_completed_role_artifact "03-contract-review.md" "contract_reviewer" "harness_contract_reviewer" ".codex/agents/harness-contract-reviewer.toml"
      require_completed_role_artifact "04-implementation-report.md" "generator" "harness_generator" ".codex/agents/harness-generator.toml"
      require_role_spawned "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_role_spawned "contract_reviewer" "harness_contract_reviewer" ".codex/agents/harness-contract-reviewer.toml"
      require_role_spawned "generator" "harness_generator" ".codex/agents/harness-generator.toml"
      ;;
    COMPLETED)
      require_file "00-input.md"
      require_completed_role_artifact "01-planner-brief.md" "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_completed_role_artifact "02-implementation-contract.md" "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_completed_role_artifact "03-contract-review.md" "contract_reviewer" "harness_contract_reviewer" ".codex/agents/harness-contract-reviewer.toml"
      require_completed_role_artifact "04-implementation-report.md" "generator" "harness_generator" ".codex/agents/harness-generator.toml"
      require_completed_role_artifact "05-evaluator-report.md" "evaluator" "harness_evaluator" ".codex/agents/harness-evaluator.toml"
      require_completed_artifact "06-final-summary.md"
      require_role_spawned "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_role_spawned "contract_reviewer" "harness_contract_reviewer" ".codex/agents/harness-contract-reviewer.toml"
      require_role_spawned "generator" "harness_generator" ".codex/agents/harness-generator.toml"
      require_role_spawned "evaluator" "harness_evaluator" ".codex/agents/harness-evaluator.toml"
      ;;
    REJECTED_FOR_REPLAN)
      require_file "00-input.md"
      require_completed_role_artifact "01-planner-brief.md" "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      require_completed_role_artifact "02-implementation-contract.md" "planner" "harness_planner" ".codex/agents/harness-planner.toml"
      ;;
    BLOCKED_FOR_EXECUTOR_UNAVAILABLE)
      [ -n "$(yaml_get blocked_reason)" ] || die "BLOCKED_FOR_EXECUTOR_UNAVAILABLE requires blocked_reason"
      grep -qF -- "- run_status: blocked" "$RUN_DIR/run-manifest.md" || die "Blocked run requires run-manifest.md run_status: blocked"
      grep -qF -- "- subagent_runtime_available: false" "$RUN_DIR/run-manifest.md" || die "Blocked run requires subagent_runtime_available: false"
      ;;
    FAILED_VERIFICATION)
      require_completed_role_artifact "05-evaluator-report.md" "evaluator" "harness_evaluator" ".codex/agents/harness-evaluator.toml"
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
  require_exact_final_status "05-evaluator-report.md" "pass fail blocked_insufficient_evidence"
  tail -n 1 "$report" | grep -qE '^- Status:[[:space:]]*pass[[:space:]]*$' || die "COMPLETED requires 05-evaluator-report.md final Status: pass"

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

validate_role_separation() {
  local planner_id contract_reviewer_id generator_id evaluator_id

  planner_id="$(role_executor_field_get planner executor_id)"
  contract_reviewer_id="$(role_executor_field_get contract_reviewer executor_id)"
  generator_id="$(role_executor_field_get generator executor_id)"
  evaluator_id="$(role_executor_field_get evaluator executor_id)"

  if [ -n "$planner_id" ] && [ -n "$contract_reviewer_id" ] && [ "$planner_id" = "$contract_reviewer_id" ]; then
    die "Contract Reviewer cannot use the same executor_id as Planner"
  fi

  if [ -n "$generator_id" ] && [ -n "$evaluator_id" ] && [ "$generator_id" = "$evaluator_id" ]; then
    die "Evaluator cannot be the same executor_id as Generator"
  fi

  for id in "$planner_id" "$contract_reviewer_id" "$generator_id" "$evaluator_id"; do
    [ "$id" != "coordinator" ] || die "Coordinator cannot be lifecycle role executor"
  done
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
require_runtime_schema
require_role_executor_templates
require_no_legacy_artifacts_or_text
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

validate_role_separation

if grep -qE 'fallback_single_session|task_tool|external_agent_session|isolated_process|production_multi_session' "$RUN_YAML" "$RUN_DIR/run-manifest.md"; then
  die "Run contains invalid fallback or non-subagent executor language"
fi

if [ "$STATE" = "APPROVED_FOR_IMPLEMENTATION" ] && [ "$APPROVED_FOR_IMPLEMENTATION" != "true" ]; then
  warn "APPROVED_FOR_IMPLEMENTATION should set approved_for_implementation: true before Generator runs"
fi

echo "OK: $RUN_DIR"
