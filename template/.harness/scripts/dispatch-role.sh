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
  bash .harness/scripts/dispatch-role.sh --reset .harness/runs/RUN-... evaluator

Create the template-based dispatch artifact for the next lifecycle role.
The coordinator must not pass a free-form role prompt to this script.
This script only writes .harness/runs/<RUN_ID>/dispatch/<role>.dispatch.md.
It does not spawn or execute the role subagent.
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

role_template() {
  case "$1" in
    planner) printf ".harness/subagents/planner.md" ;;
    contract_reviewer) printf ".harness/subagents/contract-reviewer.md" ;;
    generator) printf ".harness/subagents/generator.md" ;;
    evaluator) printf ".harness/subagents/evaluator.md" ;;
    *) return 1 ;;
  esac
}

role_output_for_state() {
  local role="$1"
  local state="$2"

  case "$role:$state" in
    planner:CREATED|planner:PLANNING) printf "01-planner-brief.md" ;;
    planner:CONTRACTING|planner:REJECTED_FOR_REPLAN) printf "02-implementation-contract.md" ;;
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

TEMPLATE_SOURCE="$(role_template "$ROLE")" || die "Invalid role: $ROLE"
TEMPLATE_PATH="$ROOT_DIR/$TEMPLATE_SOURCE"
[ -f "$TEMPLATE_PATH" ] || die "Role template not found: $TEMPLATE_SOURCE"

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

[ "$DISPATCH_MODE" = "template_based" ] || die "runtime.dispatch_mode must be template_based"
[ "$FALLBACK_ALLOWED" = "false" ] || die "runtime.fallback_allowed must be false"
[ "$SUBAGENT_RUNTIME_AVAILABLE" = "true" ] || die "Subagent runtime unavailable. Harness lifecycle requires template-based subagent orchestration."

if [ -n "$NEXT_REQUIRED_ROLE" ] && [ "$NEXT_REQUIRED_ROLE" != "$ROLE" ]; then
  die "Role $ROLE does not match next_required_role: $NEXT_REQUIRED_ROLE"
fi

[ "$(role_field_get "$ROLE" template_source)" = "$TEMPLATE_SOURCE" ] || die "role_executors.$ROLE.template_source must be $TEMPLATE_SOURCE"

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
    die "Output artifact already exists and is not reset to status: draft: $OUTPUT_ARTIFACT"
  fi
fi

mkdir -p "$RUN_DIR/dispatch"
DISPATCH_FILE="$RUN_DIR/dispatch/$ROLE.dispatch.md"
CREATED_AT="$(date -Iseconds)"

cat > "$DISPATCH_FILE" <<EOF
---
run_id: $RUN_ID
role: $ROLE
executor_type: subagent
template_source: $TEMPLATE_SOURCE
required_output_artifact: $OUTPUT_ARTIFACT
status: dispatched
created_at: $CREATED_AT
---

# Role Dispatch: $ROLE

## Required Input Artifacts

$(for input in $(required_inputs "$ROLE"); do printf -- "- %s\n" "$input"; done)

## Required Output Artifact

- $OUTPUT_ARTIFACT

## Dispatch Artifact Only

This file is an instruction artifact for the runtime executor.
\`dispatch-role.sh\` created this file only; it did not spawn, execute, or emulate the role subagent.

The runtime executor MUST consume this dispatch artifact and spawn the role-specific subagent from the template source below.

## Runtime Contract

- executor_type: subagent
- dispatch_mode: template_based
- free_form_role_prompt_allowed: false
- fallback_allowed: false
- runtime_must_spawn_template_subagent: true

## Required Final Status Line

$(case "$ROLE" in
  contract_reviewer)
    printf -- "- 03-contract-review.md must end with exactly one of:\n  - \\`- Status: approved\\`\n  - \\`- Status: rejected_requires_revision\\`\n"
    ;;
  evaluator)
    printf -- "- 05-evaluator-report.md must end with exactly one of:\n  - \\`- Status: pass\\`\n  - \\`- Status: fail\\`\n  - \\`- Status: blocked_insufficient_evidence\\`\n"
    ;;
  *)
    printf -- "- No terminal status line is required for this role artifact.\n"
    ;;
esac)

## Template Source

The runtime executor MUST instantiate the subagent from:

\`\`\`txt
$TEMPLATE_SOURCE
\`\`\`

## Forbidden Actions

- coordinator_implements_role_work
- coordinator_writes_role_artifact
- free_form_role_prompt
- bypass_subagent_template
- single_session_execution
- fallback_execution
- handoff_file_transition
EOF

echo "Created dispatch artifact:"
echo "$DISPATCH_FILE"
