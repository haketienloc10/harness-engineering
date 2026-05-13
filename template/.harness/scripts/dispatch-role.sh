#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS_DIR="$ROOT_DIR/.harness"
RUNS_DIR="$HARNESS_DIR/runs"

usage() {
  cat <<'EOF'
Usage:
  bash .harness/scripts/dispatch-role.sh .harness/runs/RUN-... planner
  bash .harness/scripts/dispatch-role.sh RUN-YYYYMMDD-NNN-task generator
  bash .harness/scripts/dispatch-role.sh --reset .harness/runs/RUN-... planner
  bash .harness/scripts/dispatch-role.sh --reset .harness/runs/RUN-... generator

Create routing metadata for the next Harness lifecycle role.
The coordinator must not pass a free-form role prompt to this script.
This script only writes .harness/runs/<RUN_ID>/dispatch/<role>.dispatch.md.
It does not spawn or execute the role subagent.
Use --reset only to redispatch Planner from REJECTED_FOR_REPLAN or Generator
from FAILED_VERIFICATION when the prior completed role artifact already exists.
EOF
}

die() {
  echo "Error: $*" >&2
  exit 1
}

RESET=0
if [ "${1:-}" = "--reset" ]; then
  RESET=1
  shift
fi

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi

[ "$#" -eq 2 ] || die "Expected run directory or run id, and role name"

RUN_INPUT="$1"
ROLE="$2"

role_agent_file() {
  case "$1" in
    planner) printf ".codex/agents/harness-planner.toml" ;;
    contract_reviewer) printf ".codex/agents/harness-contract-reviewer.toml" ;;
    generator) printf ".codex/agents/harness-generator.toml" ;;
    evaluator) printf ".codex/agents/harness-evaluator.toml" ;;
    *) return 1 ;;
  esac
}

role_agent_name() {
  case "$1" in
    planner) printf "harness_planner" ;;
    contract_reviewer) printf "harness_contract_reviewer" ;;
    generator) printf "harness_generator" ;;
    evaluator) printf "harness_evaluator" ;;
    *) return 1 ;;
  esac
}

role_output_for_state() {
  local role="$1"
  local state="$2"

  case "$role:$state" in
    planner:CREATED|planner:PLANNING|planner:REJECTED_FOR_REPLAN) printf "02-implementation-contract.md" ;;
    contract_reviewer:CONTRACT_REVIEW) printf "03-contract-review.md" ;;
    generator:APPROVED_FOR_IMPLEMENTATION|generator:GENERATING|generator:FAILED_VERIFICATION) printf "04-implementation-report.md" ;;
    evaluator:EVALUATING) printf "05-evaluator-report.md" ;;
    *) return 1 ;;
  esac
}

required_inputs() {
  local role="$1"

  case "$role" in
    planner) printf "00-input.md" ;;
    contract_reviewer) printf "00-input.md 01-planner-brief.md 02-implementation-contract.md" ;;
    generator) printf "00-input.md 01-planner-brief.md 02-implementation-contract.md 03-contract-review.md" ;;
    evaluator) printf "00-input.md 01-planner-brief.md 02-implementation-contract.md 03-contract-review.md 04-implementation-report.md" ;;
  esac
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

role_field_get() {
  local role="$1"
  local field="$2"
  awk -v role="$role" -v field="$field" '
    /^role_executors:/ { in_roles = 1; next }
    in_roles == 1 && /^[^[:space:]]/ { in_roles = 0 }
    in_roles == 1 {
      pattern = "^[[:space:]]+" role ":[[:space:]]*"
      if ($0 ~ pattern) { in_role = 1; next }
      if (in_role == 1 && $0 ~ /^  [a-zA-Z_]+:[[:space:]]*$/) { in_role = 0 }
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

artifact_field_get() {
  local file="$1"
  local field="$2"
  awk -v field="$field" '
    NR == 1 && /^---[[:space:]]*$/ { in_frontmatter = 1; next }
    in_frontmatter == 1 && /^---[[:space:]]*$/ { exit }
    in_frontmatter == 1 {
      pattern = "^" field ":[[:space:]]*"
      if ($0 ~ pattern) {
        sub(pattern, "", $0)
        gsub(/^"|"$/, "", $0)
        print $0
        exit
      }
    }
  ' "$file"
}

resolve_run_dir() {
  if [ -d "$RUN_INPUT" ]; then
    cd "$RUN_INPUT" && pwd -P
    return
  fi

  local found
  found="$(find "$RUNS_DIR" -type d -name "$RUN_INPUT" -print -quit 2>/dev/null || true)"
  [ -n "$found" ] || die "Run not found: $RUN_INPUT"
  cd "$found" && pwd -P
}

reset_command() {
  printf "bash .harness/scripts/dispatch-role.sh --reset %s %s" "$RUN_DIR" "$ROLE"
}

CODEX_AGENT_FILE="$(role_agent_file "$ROLE")" || die "Invalid role: $ROLE"
CODEX_AGENT_NAME="$(role_agent_name "$ROLE")" || die "Invalid role: $ROLE"
CODEX_AGENT_PATH="$ROOT_DIR/$CODEX_AGENT_FILE"

RUN_DIR="$(resolve_run_dir)"
RUN_YAML="$RUN_DIR/run.yaml"
[ -f "$RUN_YAML" ] || die "run.yaml not found: $RUN_YAML"

RUN_ID="$(yaml_get run_id)"
[ -n "$RUN_ID" ] || RUN_ID="$(yaml_get id)"
[ -n "$RUN_ID" ] || die "run_id is empty in run.yaml"

STATE="$(yaml_get state)"
[ -n "$STATE" ] || die "state is empty in run.yaml"
NEXT_REQUIRED_ROLE="$(yaml_get next_required_role | tr '[:upper:]' '[:lower:]' | tr ' -' '__' | sed -E 's/_+/_/g; s/^_//; s/_$//')"
DISPATCH_MODE="$(runtime_get dispatch_mode)"
FALLBACK_ALLOWED="$(runtime_get fallback_allowed)"
SUBAGENT_RUNTIME_AVAILABLE="$(runtime_get subagent_runtime_available)"

[ "$DISPATCH_MODE" = "codex_project_scoped" ] || die "runtime.dispatch_mode must be codex_project_scoped"
[ "$FALLBACK_ALLOWED" = "false" ] || die "runtime.fallback_allowed must be false"
if [ ! -f "$CODEX_AGENT_PATH" ]; then
  mkdir -p "$RUN_DIR/dispatch"
  DISPATCH_FILE="$RUN_DIR/dispatch/$ROLE.dispatch.md"
  CREATED_AT="$(date -Iseconds)"
  cat > "$DISPATCH_FILE" <<EOF
---
run_id: $RUN_ID
role: $ROLE
executor_type: subagent
required_codex_agent_name: $CODEX_AGENT_NAME
codex_agent_file: $CODEX_AGENT_FILE
status: blocked
block_code: BLOCKED_REQUIRED_CODEX_AGENT_UNAVAILABLE
blocked_reason: required_codex_agent_file_missing
created_at: $CREATED_AT
---

# Role Dispatch Blocked: $ROLE

Required Codex agent file is missing: \`$CODEX_AGENT_FILE\`.
EOF
  die "BLOCKED_REQUIRED_CODEX_AGENT_UNAVAILABLE: Required Codex agent file not found: $CODEX_AGENT_FILE"
fi

if [ "$SUBAGENT_RUNTIME_AVAILABLE" != "true" ]; then
  mkdir -p "$RUN_DIR/dispatch"
  DISPATCH_FILE="$RUN_DIR/dispatch/$ROLE.dispatch.md"
  CREATED_AT="$(date -Iseconds)"
  cat > "$DISPATCH_FILE" <<EOF
---
run_id: $RUN_ID
role: $ROLE
executor_type: subagent
required_codex_agent_name: $CODEX_AGENT_NAME
codex_agent_file: $CODEX_AGENT_FILE
status: blocked
block_code: BLOCKED_REQUIRED_SUBAGENT_UNAVAILABLE
blocked_reason: subagent_runtime_unavailable
created_at: $CREATED_AT
---

# Role Dispatch Blocked: $ROLE

Subagent runtime unavailable.
Harness lifecycle requires Codex project-scoped subagents from \`.codex/agents/\`.
This run is blocked.
No lifecycle role may be executed in this session.
EOF
  die "BLOCKED_REQUIRED_SUBAGENT_UNAVAILABLE: Subagent runtime unavailable.
Harness lifecycle requires Codex project-scoped subagents from \`.codex/agents/\`.
This run is blocked.
No lifecycle role may be executed in this session."
fi

if [ -n "$NEXT_REQUIRED_ROLE" ] && [ "$NEXT_REQUIRED_ROLE" != "$ROLE" ]; then
  die "Role $ROLE does not match next_required_role: $NEXT_REQUIRED_ROLE"
fi

[ "$(role_field_get "$ROLE" codex_agent_name)" = "$CODEX_AGENT_NAME" ] || die "role_executors.$ROLE.codex_agent_name must be $CODEX_AGENT_NAME"
[ "$(role_field_get "$ROLE" codex_agent_file)" = "$CODEX_AGENT_FILE" ] || die "role_executors.$ROLE.codex_agent_file must be $CODEX_AGENT_FILE"

OUTPUT_ARTIFACT="$(role_output_for_state "$ROLE" "$STATE")" || die "Role $ROLE is not dispatchable from state $STATE"
OUTPUT_PATH="$RUN_DIR/$OUTPUT_ARTIFACT"

for input in $(required_inputs "$ROLE"); do
  [ -f "$RUN_DIR/$input" ] || die "Required input artifact missing: $input"
done

if [ "$ROLE" = "generator" ]; then
  grep -qE '^- Status:[[:space:]]*approved[[:space:]]*$' "$RUN_DIR/03-contract-review.md" || die "Generator dispatch requires approved 03-contract-review.md"
fi

if [ -f "$OUTPUT_PATH" ]; then
  ARTIFACT_STATUS="$(artifact_field_get "$OUTPUT_PATH" status)"
  if [ "$RESET" -ne 1 ] && [ "$ARTIFACT_STATUS" != "draft" ]; then
    die "Output artifact already exists with status: $ARTIFACT_STATUS: $OUTPUT_ARTIFACT
To redispatch this lifecycle role, run exactly:
$(reset_command)"
  fi
  if [ "$RESET" -eq 1 ]; then
    case "$ROLE:$STATE" in
      planner:REJECTED_FOR_REPLAN|generator:FAILED_VERIFICATION)
        ;;
      *)
        die "--reset is only supported for planner in REJECTED_FOR_REPLAN or generator in FAILED_VERIFICATION"
        ;;
    esac
  fi
fi

mkdir -p "$RUN_DIR/dispatch"
DISPATCH_FILE="$RUN_DIR/dispatch/$ROLE.dispatch.md"
CREATED_AT="$(date -Iseconds)"

REQUIRED_INPUTS="$(required_inputs "$ROLE")"

cat > "$DISPATCH_FILE" <<EOF
---
run_id: $RUN_ID
role: $ROLE
executor_type: subagent
codex_agent_file: $CODEX_AGENT_FILE
required_codex_agent_name: $CODEX_AGENT_NAME
required_inputs: "$REQUIRED_INPUTS"
required_output_artifact: $OUTPUT_ARTIFACT
status: dispatched
created_at: $CREATED_AT
---

# Role Dispatch: $ROLE

Dispatch metadata only. This script did not spawn, execute, or emulate the role subagent.
EOF

echo "Created dispatch artifact:"
echo "$DISPATCH_FILE"
