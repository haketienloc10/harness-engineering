#!/usr/bin/env bash
set -euo pipefail

ROLE="${HARNESS_EXECUTOR_ROLE:-}"
RUN_DIR="${HARNESS_RUN_DIR:-}"
BASE_REF="${HARNESS_BASE_REF:-}"
RUN_DIR="${RUN_DIR#./}"
RUN_DIR="${RUN_DIR%/}"

if [[ "$ROLE" != "coordinator" && "$ROLE" != "orchestrator" ]]; then
  echo "SKIP role=${ROLE:-unset}"
  exit 0
fi

if [[ -z "$RUN_DIR" ]] || ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "ERR validator_misconfigured role=$ROLE run_dir=${RUN_DIR:-unset}"
  exit 2
fi

if [[ -n "$BASE_REF" ]]; then
  changed_files="$(
    {
      git diff --name-only "$BASE_REF"
      git diff --cached --name-only "$BASE_REF"
      git ls-files --others --exclude-standard
    } | sort -u
  )"
else
  changed_files="$(
    {
      git diff --name-only
      git diff --cached --name-only
      git ls-files --others --exclude-standard
    } | sort -u
  )"
fi

baseline_file="$RUN_DIR/baseline-files.txt"
if [[ -f "$baseline_file" && -n "$changed_files" ]]; then
  filtered_files=""
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    if ! grep -Fxq -- "$f" "$baseline_file"; then
      filtered_files="${filtered_files}${f}"$'\n'
    fi
  done <<< "$changed_files"
  changed_files="$(printf '%s' "$filtered_files" | sed '/^$/d')"
fi

if [[ -z "$changed_files" ]]; then
  echo "OK no_changes"
  exit 0
fi

is_allowed() {
  local p="$1"
  case "$p" in
    "$RUN_DIR/run.yaml" | \
    "$RUN_DIR/run-manifest.md" | \
    "$RUN_DIR/baseline-files.txt" | \
    "$RUN_DIR/routing-note.md" | \
    "$RUN_DIR/rework-packet.md" | \
    "$RUN_DIR/generator-rework-packet.md" | \
    "$RUN_DIR/06-final-summary.md" | \
    "$RUN_DIR/status.md" | \
    "$RUN_DIR"/routing/*.md | \
    "$RUN_DIR"/packets/*.md)
      return 0
      ;;
  esac
  return 1
}

violations=()

while IFS= read -r f; do
  [[ -z "$f" ]] && continue
  is_allowed "$f" || violations+=("$f")
done <<< "$changed_files"

if (( ${#violations[@]} > 0 )); then
  echo "FAIL BLOCKED_COORDINATOR_WRITE_SCOPE_VIOLATION"
  printf '%s\n' "${violations[@]}"
  exit 1
fi

echo "OK coordinator_write_scope"
