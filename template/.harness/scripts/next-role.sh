#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS_DIR="$ROOT_DIR/.harness"

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

write_blocked_manifest() {
  cat > "$RUN_DIR/run-manifest.md" <<'EOF'
# Run Manifest

## Execution Mode

- mode: codex_project_subagents_required
- dispatch_mode: codex_project_scoped
- fallback_allowed: false
- subagent_runtime_available: false
- run_status: blocked
- coordinator_source_edits_allowed: false
- coordinator_role_work_allowed: false

## Block Reason

Subagent runtime unavailable.
Harness lifecycle requires Codex project-scoped subagents from `.codex/agents/`.
This run is blocked.
No lifecycle role may be executed in this session.

## Required Role Instances

- planner: blocked
- contract_reviewer: blocked
- generator: blocked
- evaluator: blocked

## Role Codex Agent Files

- planner_agent_name: harness_planner
- planner_agent_file: .codex/agents/harness-planner.toml
- contract_reviewer_agent_name: harness_contract_reviewer
- contract_reviewer_agent_file: .codex/agents/harness-contract-reviewer.toml
- generator_agent_name: harness_generator
- generator_agent_file: .codex/agents/harness-generator.toml
- evaluator_agent_name: harness_evaluator
- evaluator_agent_file: .codex/agents/harness-evaluator.toml

## Required Block Codes

- required_codex_agent_unavailable: BLOCKED_REQUIRED_CODEX_AGENT_UNAVAILABLE
- required_subagent_unavailable: BLOCKED_REQUIRED_SUBAGENT_UNAVAILABLE
- required_generator_unavailable: BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
- coordinator_context_over_budget: BLOCKED_COORDINATOR_CONTEXT_OVER_BUDGET
EOF
}

set_yaml_field() {
  local key="$1"
  local value="$2"

  if grep -qE "^${key}:" "$RUN_YAML"; then
    if command -v perl >/dev/null 2>&1; then
      KEY="$key" VALUE="$value" perl -pi -e 's/^\Q$ENV{KEY}\E:.*$/$ENV{KEY}: $ENV{VALUE}/' "$RUN_YAML"
    else
      sed -i "s/^${key}:.*$/${key}: ${value}/" "$RUN_YAML"
    fi
  else
    printf "%s: %s\n" "$key" "$value" >> "$RUN_YAML"
  fi
}

role_executor_get() {
  local role="$1"
  awk -v role="$role" '
    /^role_executors:/ { in_roles = 1; next }
    in_roles == 1 && /^[^[:space:]]/ { in_roles = 0 }
    in_roles == 1 {
      pattern = "^[[:space:]]+" role ":[[:space:]]*"
      if ($0 ~ pattern) {
        value = $0
        sub(pattern, "", value)
        gsub(/^"|"$/, "", value)
        if (value != "") {
          print value
          exit
        }
        in_role = 1
        next
      }
      if (in_role == 1 && $0 ~ /^  [a-zA-Z_]+:[[:space:]]*$/) {
        in_role = 0
      }
      if (in_role == 1 && $0 ~ /^[[:space:]]+executor_type:[[:space:]]*/) {
        sub(/^[[:space:]]+executor_type:[[:space:]]*/, "", $0)
        gsub(/^"|"$/, "", $0)
        print $0
        exit
      }
    }
  ' "$RUN_YAML"
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

artifact_to_role() {
  case "$1" in
    01-planner-brief.md|02-implementation-contract.md)
      printf "planner"
      ;;
    03-contract-review.md)
      printf "contract_reviewer"
      ;;
    04-implementation-report.md)
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
    CREATED|PLANNING|REJECTED_FOR_REPLAN)
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
    BLOCKED_FOR_EXECUTOR_UNAVAILABLE)
      artifact_to_role "$NEXT_REQUIRED_ARTIFACT"
      ;;
    COMPLETED|CANCELLED)
      printf "none"
      ;;
  esac
}

state_required_artifact() {
  case "$1" in
    CREATED|PLANNING|REJECTED_FOR_REPLAN)
      printf "02-implementation-contract.md"
      ;;
    CONTRACT_REVIEW)
      printf "03-contract-review.md"
      ;;
    APPROVED_FOR_IMPLEMENTATION|GENERATING)
      printf "04-implementation-report.md"
      ;;
    EVALUATING)
      printf "05-evaluator-report.md"
      ;;
    FAILED_VERIFICATION)
      printf "04-implementation-report.md"
      ;;
    BLOCKED_FOR_EXECUTOR_UNAVAILABLE)
      printf "%s" "${NEXT_REQUIRED_ARTIFACT:-none}"
      ;;
    COMPLETED|CANCELLED)
      printf "none"
      ;;
  esac
}

role_key_to_executor() {
  case "$1" in
    planner)
      printf "planner"
      ;;
    contract_reviewer)
      printf "contract-reviewer"
      ;;
    generator)
      printf "generator"
      ;;
    evaluator)
      printf "evaluator"
      ;;
    *)
      printf "none"
      ;;
  esac
}

block_for_executor_unavailable() {
  if command -v bash >/dev/null 2>&1 && [ -f "$HARNESS_DIR/scripts/set-runtime-capability.sh" ]; then
    bash "$HARNESS_DIR/scripts/set-runtime-capability.sh" "$RUN_DIR" false "Subagent runtime unavailable. Harness lifecycle requires Codex project-scoped subagents from .codex/agents/. This run is blocked. No lifecycle role may be executed in this session." >/dev/null
    STATE="BLOCKED_FOR_EXECUTOR_UNAVAILABLE"
    return
  fi

  set_yaml_field "state" "BLOCKED_FOR_EXECUTOR_UNAVAILABLE"
  set_yaml_field "blocked_reason" "\"Subagent runtime unavailable. Harness lifecycle requires Codex project-scoped subagents from .codex/agents/. This run is blocked. No lifecycle role may be executed in this session.\""
  write_blocked_manifest
  STATE="BLOCKED_FOR_EXECUTOR_UNAVAILABLE"
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
SUBAGENT_RUNTIME_AVAILABLE="$(runtime_get subagent_runtime_available)"
DISPATCH_MODE="$(runtime_get dispatch_mode)"
FALLBACK_ALLOWED="$(runtime_get fallback_allowed)"
APPROVED_FOR_IMPLEMENTATION="$(yaml_get approved_for_implementation)"
GENERATOR_ALLOWED="$(yaml_get generator_allowed)"

[ -n "$STATE" ] || die "state is empty in run.yaml"
recognized_state "$STATE" || die "Unrecognized lifecycle state: $STATE"

if [ "$STATE" = "APPROVED_FOR_IMPLEMENTATION" ] || [ "$STATE" = "GENERATING" ]; then
  [ "$APPROVED_FOR_IMPLEMENTATION" = "true" ] || die "$STATE requires approved_for_implementation: true"
  [ "$GENERATOR_ALLOWED" = "true" ] || die "$STATE requires generator_allowed: true"
fi

NEXT_ROLE="$(state_next_role "$STATE")"
NEXT_EXECUTOR="$(role_key_to_executor "$NEXT_ROLE")"
REQUIRED_ARTIFACT="$(state_required_artifact "$STATE")"
BLOCKED="false"

if [ "$NEXT_ROLE" != "none" ]; then
  if [ "$STATE" = "BLOCKED_FOR_EXECUTOR_UNAVAILABLE" ]; then
    BLOCKED="true"
  elif [ "$DISPATCH_MODE" != "codex_project_scoped" ]; then
    block_for_executor_unavailable
    BLOCKED="true"
  elif [ "$FALLBACK_ALLOWED" != "false" ]; then
    block_for_executor_unavailable
    BLOCKED="true"
  elif [ "$SUBAGENT_RUNTIME_AVAILABLE" != "true" ]; then
    block_for_executor_unavailable
    BLOCKED="true"
  fi
fi

printf "RUN_DIR=%s\n" "$RUN_DIR"
printf "STATE=%s\n" "$STATE"
printf "NEXT_ROLE=%s\n" "$NEXT_ROLE"
printf "NEXT_EXECUTOR=%s\n" "$NEXT_EXECUTOR"
printf "REQUIRED_ARTIFACT=%s\n" "$REQUIRED_ARTIFACT"
printf "BLOCKED=%s\n" "$BLOCKED"
if [ "$BLOCKED" = "true" ]; then
  cat <<'EOF'
Subagent runtime unavailable.
Harness lifecycle requires Codex project-scoped subagents from `.codex/agents/`.
This run is blocked.
No lifecycle role may be executed in this session.
EOF
fi
