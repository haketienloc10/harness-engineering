#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-all}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

case "$MODE" in
  all|state|distribution) ;;
  *) printf 'usage: %s [all|state|distribution]\n' "$0" >&2; exit 2 ;;
esac

cd "$ROOT"

test "$(uname -s)/$(uname -m)" = "Linux/x86_64"
test -x target/debug/harness-cli
test -x _harness/bin/harness-cli

run_state_qualification() {
  cargo test -p harness-cli \
    infrastructure::tests::doctor_reports_a_legacy_version_one_database_as_behind
  cargo test -p harness-cli \
    infrastructure::tests::migrate_applies_story_verify_columns_to_existing_database
  cargo test -p harness-cli \
    infrastructure::tests::doctor_detects_ahead_database_and_never_mutates_it
  cargo test -p harness-cli \
    infrastructure::tests::backup_can_restore_the_pre_migration_database
  cargo test -p harness-cli \
    infrastructure::tests::task_session_lease_guards_session_story_and_worktree_concurrency
  cargo test -p harness-cli \
    infrastructure::tests::phase4_failure_matrix_is_structured_and_preflight_is_non_mutating

  local clone="$WORK/fresh-clone"
  local candidate_patch="$WORK/candidate.patch"
  git clone --quiet --no-local "$ROOT" "$clone"
  git diff --binary HEAD >"$candidate_patch"
  if test -s "$candidate_patch"; then
    git -C "$clone" apply "$candidate_patch"
  fi
  while IFS= read -r -d '' path; do
    mkdir -p "$clone/$(dirname "$path")"
    cp "$ROOT/$path" "$clone/$path"
  done < <(git ls-files --others --exclude-standard -z)
  git -C "$clone" add -A
  if ! git -C "$clone" diff --cached --quiet; then
    git -C "$clone" -c user.name='CL-70 Qualification' \
      -c user.email='cl70@example.invalid' commit --quiet -m 'CL-70 release candidate'
  fi

  local clone_cli="$clone/_harness/bin/harness-cli"
  local rebuild_one='.harness-evidence/cl70-rebuild-one.db'
  local rebuild_two='.harness-evidence/cl70-rebuild-two.db'
  mkdir -p "$clone/.harness-evidence"
  HARNESS_REPO_ROOT="$clone" "$clone_cli" memory check --dry-run --json \
    >"$WORK/memory-check.json"
  (cd "$clone" && HARNESS_REPO_ROOT="$clone" "$clone_cli" memory rebuild --dry-run \
    --output "$rebuild_one" --json) >"$WORK/rebuild-one.json"
  (cd "$clone" && HARNESS_REPO_ROOT="$clone" "$clone_cli" memory rebuild --dry-run \
    --output "$rebuild_two" --json) >"$WORK/rebuild-two.json"
  test "$(jq -r .logical_digest "$WORK/rebuild-one.json")" = \
    "$(jq -r .logical_digest "$WORK/rebuild-two.json")"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$clone/$rebuild_one" \
    "$clone_cli" doctor --strict --json | grep -q '"code":"HEALTHY"'

  find docs/tasks -type f -print0 | sort -z | xargs -0 sha256sum \
    >"$WORK/source-capsules.sha256"
  (cd "$clone" && find docs/tasks -type f -print0 | sort -z | xargs -0 sha256sum) \
    >"$WORK/clone-capsules.sha256"
  cmp "$WORK/source-capsules.sha256" "$WORK/clone-capsules.sha256"

  local repository_id
  repository_id="$(cat "$clone/.harness-id")"
  git -C "$clone" switch --quiet -c cl70-branch-switch
  test "$(cat "$clone/.harness-id")" = "$repository_id"
  printf 'dirty release qualification\n' >"$clone/cl70-dirty-probe.txt"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$clone/$rebuild_one" \
    "$clone_cli" doctor --strict --json | grep -q '"code":"HEALTHY"'

  local first_task second_task
  first_task="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$clone/$rebuild_one" \
    "$clone_cli" task start --type 'maintenance request' \
    --summary 'CL-70 concurrent session A' --owner codex --session cl70-a \
    --behavior-bearing no --json | jq -r .task_id)"
  if HARNESS_REPO_ROOT="$clone" HARNESS_DB="$clone/$rebuild_one" \
    "$clone_cli" task start --type 'maintenance request' \
    --summary 'CL-70 concurrent session B conflict' --owner codex --session cl70-b \
    --behavior-bearing no --json >"$WORK/session-conflict.out" 2>&1; then
    echo 'second live session unexpectedly acquired the same worktree' >&2
    exit 1
  fi
  grep -q 'TASK_OWNERSHIP_CONFLICT' "$WORK/session-conflict.out"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$clone/$rebuild_one" \
    "$clone_cli" task block --id "$first_task" --owner codex --session cl70-a \
      --reason 'release qualification fixture' --json \
    >/dev/null
  second_task="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$clone/$rebuild_one" \
    "$clone_cli" task start --type 'maintenance request' \
    --summary 'CL-70 concurrent session B' --owner codex --session cl70-b \
    --behavior-bearing no --json | jq -r .task_id)"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$clone/$rebuild_one" \
    "$clone_cli" task block --id "$second_task" --owner codex --session cl70-b \
      --reason 'release qualification fixture' --json \
    >/dev/null
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$clone/$rebuild_one" \
    "$clone_cli" task resume --id "$first_task" --owner codex --session cl70-a --json \
    | grep -q '"status":"in_progress"'

  printf 'state qualification: ok; rebuild_digest=%s; capsules=%s\n' \
    "$(jq -r .logical_digest "$WORK/rebuild-one.json")" \
    "$(find docs/tasks -type f | wc -l)"
}

run_distribution_qualification() {
  bash tests/installer_state_safety.sh
  target/debug/harness-cli workflow parity --json | grep -q 'WORKFLOW_PARITY_OK'
  _harness/bin/harness-cli workflow parity --json | grep -q 'WORKFLOW_PARITY_OK'

  target/debug/harness-cli workflow commands >"$WORK/source-commands.txt"
  _harness/bin/harness-cli workflow commands >"$WORK/packaged-commands.txt"
  grep -v '^#' _harness/command-manifest.txt | sed '/^[[:space:]]*$/d' \
    >"$WORK/tracked-commands.txt"
  cmp "$WORK/source-commands.txt" "$WORK/packaged-commands.txt"
  cmp "$WORK/source-commands.txt" "$WORK/tracked-commands.txt"

  local samples=30
  local index start_ns elapsed_ns
  : >"$WORK/startup-us.txt"
  for ((index=0; index<samples; index++)); do
    start_ns="$(date +%s%N)"
    _harness/bin/harness-cli --version >/dev/null
    elapsed_ns=$(( $(date +%s%N) - start_ns ))
    printf '%s\n' "$((elapsed_ns / 1000))" >>"$WORK/startup-us.txt"
  done
  sort -n "$WORK/startup-us.txt" >"$WORK/startup-us.sorted"
  local p50 p95
  p50="$(sed -n "$((samples * 50 / 100))p" "$WORK/startup-us.sorted")"
  p95="$(sed -n "$((samples * 95 / 100))p" "$WORK/startup-us.sorted")"
  test "$p95" -lt 500000

  printf 'distribution qualification: ok; startup_p50_us=%s; startup_p95_us=%s; command_count=%s\n' \
    "$p50" "$p95" "$(wc -l <"$WORK/source-commands.txt")"
}

case "$MODE" in
  state) run_state_qualification ;;
  distribution) run_distribution_qualification ;;
  all)
    run_state_qualification
    run_distribution_qualification
    ;;
esac
