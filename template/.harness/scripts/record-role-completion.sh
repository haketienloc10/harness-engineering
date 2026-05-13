#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash .harness/scripts/record-role-completion.sh .harness/runs/RUN-... planner EXECUTOR_ID
  bash .harness/scripts/record-role-completion.sh .harness/runs/RUN-... generator EXECUTOR_ID

Record completed subagent metadata in run.yaml, run-manifest.md, and the role-owned artifact frontmatter.
EOF
}

die() {
  echo "Error: $*" >&2
  exit 1
}

[ "${1:-}" != "--help" ] && [ "${1:-}" != "-h" ] || {
  usage
  exit 0
}

[ "$#" -eq 3 ] || die "Expected run directory, role, and executor_id"

RUN_DIR="${1%/}"
ROLE="$2"
EXECUTOR_ID="$3"
RUN_YAML="$RUN_DIR/run.yaml"
RUN_MANIFEST="$RUN_DIR/run-manifest.md"

[ -d "$RUN_DIR" ] || die "Run directory not found: $RUN_DIR"
[ -f "$RUN_YAML" ] || die "run.yaml not found: $RUN_YAML"
[ -f "$RUN_MANIFEST" ] || die "run-manifest.md not found: $RUN_MANIFEST"
[ -n "$EXECUTOR_ID" ] || die "executor_id must not be empty"
[ "$EXECUTOR_ID" != "coordinator" ] || die "coordinator cannot be lifecycle role executor_id"

role_agent_name() {
  case "$1" in
    planner) printf "harness_planner" ;;
    contract_reviewer) printf "harness_contract_reviewer" ;;
    generator) printf "harness_generator" ;;
    evaluator) printf "harness_evaluator" ;;
    *) return 1 ;;
  esac
}

role_agent_file() {
  case "$1" in
    planner) printf ".codex/agents/harness-planner.toml" ;;
    contract_reviewer) printf ".codex/agents/harness-contract-reviewer.toml" ;;
    generator) printf ".codex/agents/harness-generator.toml" ;;
    evaluator) printf ".codex/agents/harness-evaluator.toml" ;;
    *) return 1 ;;
  esac
}

role_artifacts() {
  case "$1" in
    planner) printf "01-planner-brief.md 02-implementation-contract.md" ;;
    contract_reviewer) printf "03-contract-review.md" ;;
    generator) printf "04-implementation-report.md" ;;
    evaluator) printf "05-evaluator-report.md" ;;
    *) return 1 ;;
  esac
}

CODEX_AGENT_NAME="$(role_agent_name "$ROLE")" || die "Invalid role: $ROLE"
CODEX_AGENT_FILE="$(role_agent_file "$ROLE")" || die "Invalid role: $ROLE"
ARTIFACTS="$(role_artifacts "$ROLE")" || die "Invalid role: $ROLE"

update_yaml_role_field() {
  local field="$1"
  local value="$2"
  awk -v role="$ROLE" -v field="$field" -v value="$value" '
    /^role_executors:/ { in_roles = 1; print; next }
    in_roles == 1 && /^[^[:space:]]/ { in_roles = 0 }
    in_roles == 1 {
      role_pattern = "^[[:space:]]+" role ":[[:space:]]*"
      if ($0 ~ role_pattern) { in_role = 1; print; next }
      if (in_role == 1 && $0 ~ /^  [a-zA-Z_]+:[[:space:]]*$/) { in_role = 0 }
      if (in_role == 1) {
        field_pattern = "^[[:space:]]+" field ":[[:space:]]*"
        if ($0 ~ field_pattern) {
          print "    " field ": " value
          next
        }
      }
    }
    { print }
  ' "$RUN_YAML" > "$RUN_YAML.tmp.$$"
  mv "$RUN_YAML.tmp.$$" "$RUN_YAML"
}

update_manifest_role() {
  awk -v role="$ROLE" '
    {
      pattern = "^- " role ":[[:space:]]*"
      if ($0 ~ pattern) {
        print "- " role ": completed"
      } else {
        print
      }
    }
  ' "$RUN_MANIFEST" > "$RUN_MANIFEST.tmp.$$"
  mv "$RUN_MANIFEST.tmp.$$" "$RUN_MANIFEST"
}

update_artifact_field() {
  local artifact="$1"
  local field="$2"
  local value="$3"
  awk -v field="$field" -v value="$value" '
    NR == 1 && /^---[[:space:]]*$/ { in_frontmatter = 1; print; next }
    in_frontmatter == 1 && /^---[[:space:]]*$/ { in_frontmatter = 0; print; next }
    in_frontmatter == 1 {
      pattern = "^" field ":[[:space:]]*"
      if ($0 ~ pattern) {
        print field ": " value
        next
      }
    }
    { print }
  ' "$RUN_DIR/$artifact" > "$RUN_DIR/$artifact.tmp.$$"
  mv "$RUN_DIR/$artifact.tmp.$$" "$RUN_DIR/$artifact"
}

update_yaml_role_field "executor_type" "subagent"
update_yaml_role_field "executor_id" "$EXECUTOR_ID"
update_yaml_role_field "codex_agent_name" "$CODEX_AGENT_NAME"
update_yaml_role_field "codex_agent_file" "$CODEX_AGENT_FILE"
update_yaml_role_field "status" "completed"
update_manifest_role

for artifact in $ARTIFACTS; do
  [ -f "$RUN_DIR/$artifact" ] || die "Role artifact missing: $artifact"
  update_artifact_field "$artifact" "role" "$ROLE"
  update_artifact_field "$artifact" "executor_type" "subagent"
  update_artifact_field "$artifact" "executor_id" "$EXECUTOR_ID"
  update_artifact_field "$artifact" "codex_agent_name" "$CODEX_AGENT_NAME"
  update_artifact_field "$artifact" "codex_agent_file" "$CODEX_AGENT_FILE"
  update_artifact_field "$artifact" "status" "completed"
  if [ "$ROLE" = "evaluator" ]; then
    update_artifact_field "$artifact" "evaluator_executor_id" "$EXECUTOR_ID"
  fi
done

echo "OK role_completed role=$ROLE executor_id=$EXECUTOR_ID"
