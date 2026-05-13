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

update_manifest_field() {
  local field="$1"
  local value="$2"
  awk -v field="$field" -v value="$value" '
    {
      pattern = "^- " field ":[[:space:]]*"
      if ($0 ~ pattern) {
        print "- " field ": " value
      } else {
        print
      }
    }
  ' "$RUN_MANIFEST" > "$RUN_MANIFEST.tmp.$$"
  mv "$RUN_MANIFEST.tmp.$$" "$RUN_MANIFEST"
}

update_yaml_field() {
  local field="$1"
  local value="$2"
  awk -v field="$field" -v value="$value" '
    {
      pattern = "^" field ":[[:space:]]*"
      if ($0 ~ pattern) {
        print field ": " value
      } else {
        print
      }
    }
  ' "$RUN_YAML" > "$RUN_YAML.tmp.$$"
  mv "$RUN_YAML.tmp.$$" "$RUN_YAML"
}

update_artifact_field() {
  local artifact="$1"
  local field="$2"
  local value="$3"
  awk -v field="$field" -v value="$value" '
    NR == 1 && /^---[[:space:]]*$/ { in_frontmatter = 1; print; next }
    in_frontmatter == 1 && /^---[[:space:]]*$/ {
      if (found != 1) {
        print field ": " value
      }
      in_frontmatter = 0
      print
      next
    }
    in_frontmatter == 1 {
      pattern = "^" field ":[[:space:]]*"
      if ($0 ~ pattern) {
        print field ": " value
        found = 1
        next
      }
    }
    { print }
  ' "$RUN_DIR/$artifact" > "$RUN_DIR/$artifact.tmp.$$"
  mv "$RUN_DIR/$artifact.tmp.$$" "$RUN_DIR/$artifact"
}

artifact_field_get() {
  local artifact="$1"
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
  ' "$RUN_DIR/$artifact"
}

final_status() {
  local artifact="$1"
  local status
  status="$(tail -n 1 "$RUN_DIR/$artifact" | sed -n -E 's/^- Status:[[:space:]]*([^[:space:]]+)[[:space:]]*$/\1/p')"
  [ -n "$status" ] || die "$artifact must end with final line: - Status: <status>"
  printf "%s" "$status"
}

apply_lifecycle_transition() {
  local state="$1"
  local approved="$2"
  local generator="$3"
  local next_role="$4"
  local next_executor="$5"
  local next_artifact="$6"
  local last_artifact="$7"
  local run_status="$8"

  update_yaml_field "state" "$state"
  update_yaml_field "approved_for_implementation" "$approved"
  update_yaml_field "generator_allowed" "$generator"
  update_yaml_field "next_required_role" "$next_role"
  update_yaml_field "next_required_executor" "$next_executor"
  update_yaml_field "next_required_artifact" "$next_artifact"
  update_yaml_field "last_artifact" "$last_artifact"
  update_manifest_field "run_status" "$run_status"
}

for artifact in $ARTIFACTS; do
  [ -f "$RUN_DIR/$artifact" ] || die "Role artifact missing: $artifact"
done

NEXT_STATE=""
NEXT_APPROVED="false"
NEXT_GENERATOR_ALLOWED="false"
NEXT_ROLE="null"
NEXT_EXECUTOR="none"
NEXT_ARTIFACT=""
LAST_ARTIFACT=""
RUN_STATUS=""
GENERATOR_EXECUTOR_ID=""

case "$ROLE" in
  planner)
    NEXT_STATE="CONTRACT_REVIEW"
    NEXT_ROLE="contract_reviewer"
    NEXT_EXECUTOR="contract_reviewer"
    NEXT_ARTIFACT="03-contract-review.md"
    LAST_ARTIFACT="02-implementation-contract.md"
    RUN_STATUS="contract_review"
    ;;
  contract_reviewer)
    case "$(final_status "03-contract-review.md")" in
      approved)
        NEXT_STATE="APPROVED_FOR_IMPLEMENTATION"
        NEXT_APPROVED="true"
        NEXT_GENERATOR_ALLOWED="true"
        NEXT_ROLE="generator"
        NEXT_EXECUTOR="generator"
        NEXT_ARTIFACT="04-implementation-report.md"
        LAST_ARTIFACT="03-contract-review.md"
        RUN_STATUS="approved_for_implementation"
        ;;
      rejected_requires_revision)
        NEXT_STATE="REJECTED_FOR_REPLAN"
        NEXT_ROLE="planner"
        NEXT_EXECUTOR="planner"
        NEXT_ARTIFACT="02-implementation-contract.md"
        LAST_ARTIFACT="03-contract-review.md"
        RUN_STATUS="rejected_for_replan"
        ;;
      *)
        die "03-contract-review.md final status must be approved or rejected_requires_revision"
        ;;
    esac
    ;;
  generator)
    NEXT_STATE="EVALUATING"
    NEXT_APPROVED="true"
    NEXT_GENERATOR_ALLOWED="true"
    NEXT_ROLE="evaluator"
    NEXT_EXECUTOR="evaluator"
    NEXT_ARTIFACT="05-evaluator-report.md"
    LAST_ARTIFACT="04-implementation-report.md"
    RUN_STATUS="evaluating"
    ;;
  evaluator)
    GENERATOR_EXECUTOR_ID="$(artifact_field_get "04-implementation-report.md" "executor_id")"
    [ -n "$GENERATOR_EXECUTOR_ID" ] || die "04-implementation-report.md missing executor_id"
    [ "$EXECUTOR_ID" != "$GENERATOR_EXECUTOR_ID" ] || die "Evaluator cannot use same executor_id as Generator: $EXECUTOR_ID"
    case "$(final_status "05-evaluator-report.md")" in
      pass)
        NEXT_STATE="COMPLETED"
        NEXT_APPROVED="true"
        NEXT_GENERATOR_ALLOWED="true"
        NEXT_ROLE="null"
        NEXT_EXECUTOR="none"
        NEXT_ARTIFACT="06-final-summary.md"
        LAST_ARTIFACT="05-evaluator-report.md"
        RUN_STATUS="completed"
        ;;
      fail|blocked_insufficient_evidence)
        NEXT_STATE="FAILED_VERIFICATION"
        NEXT_APPROVED="true"
        NEXT_GENERATOR_ALLOWED="true"
        NEXT_ROLE="generator"
        NEXT_EXECUTOR="generator"
        NEXT_ARTIFACT="04-implementation-report.md"
        LAST_ARTIFACT="05-evaluator-report.md"
        RUN_STATUS="failed_verification"
        ;;
      *)
        die "05-evaluator-report.md final status must be pass, fail, or blocked_insufficient_evidence"
        ;;
    esac
    ;;
esac

update_yaml_role_field "executor_type" "subagent"
update_yaml_role_field "executor_id" "$EXECUTOR_ID"
update_yaml_role_field "codex_agent_name" "$CODEX_AGENT_NAME"
update_yaml_role_field "codex_agent_file" "$CODEX_AGENT_FILE"
update_yaml_role_field "status" "completed"
update_manifest_role
apply_lifecycle_transition "$NEXT_STATE" "$NEXT_APPROVED" "$NEXT_GENERATOR_ALLOWED" "$NEXT_ROLE" "$NEXT_EXECUTOR" "$NEXT_ARTIFACT" "$LAST_ARTIFACT" "$RUN_STATUS"

for artifact in $ARTIFACTS; do
  update_artifact_field "$artifact" "role" "$ROLE"
  update_artifact_field "$artifact" "executor_type" "subagent"
  update_artifact_field "$artifact" "executor_id" "$EXECUTOR_ID"
  update_artifact_field "$artifact" "codex_agent_name" "$CODEX_AGENT_NAME"
  update_artifact_field "$artifact" "codex_agent_file" "$CODEX_AGENT_FILE"
  update_artifact_field "$artifact" "status" "completed"
  if [ "$ROLE" = "evaluator" ]; then
    update_artifact_field "$artifact" "generator_executor_id" "$GENERATOR_EXECUTOR_ID"
    update_artifact_field "$artifact" "evaluator_executor_id" "$EXECUTOR_ID"
    update_artifact_field "$artifact" "same_executor_as_generator" "false"
  fi
done

echo "OK role_completed role=$ROLE executor_id=$EXECUTOR_ID"
