#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash .harness/scripts/set-runtime-capability.sh .harness/runs/RUN-... true
  bash .harness/scripts/set-runtime-capability.sh .harness/runs/RUN-... false "reason"

Record a manual runtime capability assertion for the current run.
This is not a proof that Codex can spawn agents; it records the coordinator's
runtime capability decision in run.yaml and run-manifest.md.
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

[ "$#" -ge 2 ] && [ "$#" -le 3 ] || die "Expected run directory, true|false, and optional reason"

RUN_DIR="${1%/}"
AVAILABLE="$2"
REASON="${3:-}"
RUN_YAML="$RUN_DIR/run.yaml"
RUN_MANIFEST="$RUN_DIR/run-manifest.md"

[ -d "$RUN_DIR" ] || die "Run directory not found: $RUN_DIR"
[ -f "$RUN_YAML" ] || die "run.yaml not found: $RUN_YAML"
[ -f "$RUN_MANIFEST" ] || die "run-manifest.md not found: $RUN_MANIFEST"

case "$AVAILABLE" in
  true|false)
    ;;
  *)
    die "Availability must be true or false"
    ;;
esac

replace_yaml_runtime_field() {
  local field="$1"
  local value="$2"
  awk -v field="$field" -v value="$value" '
    /^runtime:/ { in_runtime = 1; print; next }
    in_runtime == 1 && /^[^[:space:]]/ { in_runtime = 0 }
    in_runtime == 1 {
      pattern = "^[[:space:]]+" field ":[[:space:]]*"
      if ($0 ~ pattern) {
        sub(pattern ".*$", "  " field ": " value)
      }
    }
    { print }
  ' "$RUN_YAML" > "$RUN_YAML.tmp.$$"
  mv "$RUN_YAML.tmp.$$" "$RUN_YAML"
}

replace_manifest_field() {
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

replace_root_yaml_field() {
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

replace_yaml_runtime_field "subagent_runtime_available" "$AVAILABLE"
replace_manifest_field "subagent_runtime_available" "$AVAILABLE"

if [ "$AVAILABLE" = "true" ]; then
  replace_manifest_field "run_status" "ready_for_planner_dispatch"
  echo "OK subagent_runtime_available=true"
else
  replace_manifest_field "run_status" "blocked"
  replace_root_yaml_field "state" "BLOCKED_FOR_EXECUTOR_UNAVAILABLE"
  if [ -n "$REASON" ]; then
    replace_root_yaml_field "blocked_reason" "\"$REASON\""
  else
    replace_root_yaml_field "blocked_reason" "\"Subagent runtime unavailable. Harness lifecycle requires Codex project-scoped subagents from .codex/agents/. This run is blocked. No lifecycle role may be executed in this session.\""
  fi
  echo "OK subagent_runtime_available=false"
fi
