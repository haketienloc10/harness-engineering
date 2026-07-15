#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-all}"
CLI="${HARNESS_CLI:-$ROOT/target/debug/harness-cli}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

case "$MODE" in
  all|baseline|command|memory|audit) ;;
  *) printf 'usage: %s [all|baseline|command|memory|audit]\n' "$0" >&2; exit 2 ;;
esac

test -x "$CLI"
command -v jq >/dev/null
cd "$ROOT"

new_clone() {
  local name="$1"
  local base="$WORK/base-clone"
  local clone="$WORK/$name"
  if ! test -d "$base/.git"; then
    git clone --quiet --no-local "$ROOT" "$base"
    git diff --binary HEAD | git -C "$base" apply --whitespace=nowarn -
    while IFS= read -r -d '' path; do
      mkdir -p "$base/$(dirname "$path")"
      cp -a "$ROOT/$path" "$base/$path"
    done < <(git ls-files --others --exclude-standard -z)
  fi
  cp -a "$base" "$clone"
  printf '%s\n' "$clone"
}

init_database() {
  local clone="$1"
  local database="$2"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" init >/dev/null
}

probe_minimal_start() {
  local clone database
  clone="$(new_clone minimal-start)"
  database="$clone/minimal-start.db"
  init_database "$clone" "$database"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type change-request --summary "Add account export" \
    >"$WORK/minimal-start.out" 2>"$WORK/minimal-start.err"
}

probe_start_json_contract() {
  local clone database
  clone="$(new_clone start-json)"
  database="$clone/start-json.db"
  init_database "$clone" "$database"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type change_request --summary "Add account export" --json \
    | jq -e '
      .ok == true and
      (.task_id | type == "string") and
      (.status | type == "string") and
      has("lane") and has("lane_reasons") and
      has("behavior") and has("requirements") and
      (.context | has("must_read") and has("should_read") and has("skip") and
        has("stop_condition")) and
      has("proof_gates") and has("completion_gates") and
      has("relevant_tools") and
      (.next_command | type == "string" and length > 0)
    ' >/dev/null
}

probe_status_json_contract() {
  local clone database task_id
  clone="$(new_clone status-json)"
  database="$clone/status-json.db"
  init_database "$clone" "$database"
  task_id="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type 'maintenance request' --summary 'Status schema fixture' \
    --owner codex --session clp001-r1-status --behavior-bearing no --json \
    | jq -r .task_id)"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task status \
    --id "$task_id" --json \
    | jq -e '
      has("status") and has("ownership") and has("session") and
      has("worktree") and has("lease") and has("transitions") and
      has("links") and has("friction") and has("gates") and
      (.proof | has("layers") and has("freshness")) and
      (.context | has("must_read") and has("should_read") and has("skip")) and
      has("capsule") and
      (.remediation | type == "array")
    ' >/dev/null
}

probe_json_error_envelope() {
  local output status
  if output="$("$CLI" task start --type invalid --summary invalid \
    --behavior-bearing no --json 2>&1)"; then
    status=0
  else
    status=$?
  fi
  test "$status" -eq 2
  jq -e '
    .ok == false and
    (.code | type == "string" and length > 0) and
    (.message | type == "string" and length > 0) and
    has("details") and
    (.remediation | type == "array")
  ' <<<"$output" >/dev/null
}

probe_json_parse_error_envelope() {
  local output status
  if output="$("$CLI" task start --type change-request --json 2>&1)"; then
    status=0
  else
    status=$?
  fi
  test "$status" -eq 2
  jq -e '
    .ok == false and .code == "CLI_USAGE_ERROR" and
    (.message | type == "string" and length > 0) and
    (.details | has("kind")) and (.remediation | length > 0)
  ' <<<"$output" >/dev/null
}

probe_json_preflight_error_envelope() {
  local clone output status
  clone="$(new_clone preflight-json)"
  if output="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$clone/missing.db" \
    "$CLI" task status --id TASK-000001 --json 2>&1)"; then
    status=0
  else
    status=$?
  fi
  test "$status" -eq 3
  jq -e '
    .ok == false and .code == "DB_MISSING" and
    (.message | length > 0) and (.details | type == "object") and
    (.remediation | length > 0)
  ' <<<"$output" >/dev/null
}

probe_json_policy_error_envelope() {
  local clone database output status
  clone="$(new_clone policy-json)"
  database="$clone/policy-json.db"
  init_database "$clone" "$database"
  if output="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type change-request --summary 'Reject policy downgrade' --flags auth \
    --lane tiny --lane-reason fixture --behavior-bearing no --json 2>&1)"; then
    status=0
  else
    status=$?
  fi
  test "$status" -eq 2
  jq -e '
    .ok == false and (.code | type == "string" and length > 0) and
    (.message | contains("cannot lower policy lane")) and
    (.details | type == "object") and (.remediation | length > 0)
  ' <<<"$output" >/dev/null
}

probe_auto_classification_is_typed() {
  local clone index database output
  clone="$(new_clone auto-classification)"
  index=0
  for input in 'change request' change_request change-request; do
    index=$((index + 1))
    database="$clone/auto-$index.db"
    init_database "$clone" "$database"
    output="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
      --type "$input" --summary 'Documentation wording must not control behavior' --json)"
    jq -e '
      .behavior.mode == "auto" and .behavior.bearing == true and
      .behavior.summary_inspected == false and
      (.behavior.reasons | index("typed-input:change_request")) != null
    ' <<<"$output" >/dev/null
  done
  database="$clone/auto-maintenance.db"
  init_database "$clone" "$database"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type maintenance --summary 'Add behavior words that must be ignored' --json \
    | jq -e '
      .behavior.mode == "auto" and .behavior.bearing == false and
      .behavior.summary_inspected == false and
      (.behavior.reasons | index("typed-input:maintenance")) != null
    ' >/dev/null
}

probe_minimal_finish_executes() {
  local clone database task_id intake_id
  clone="$(new_clone minimal-finish)"
  database="$clone/harness.db"
  init_database "$clone" "$database"
  task_id="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type maintenance --summary 'Minimal finish fixture' \
    --owner codex --session minimal-finish --behavior-bearing no --json | jq -r .task_id)"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task context acknowledge \
    --id "$task_id" --read '<changed-files>' --actor codex --json >/dev/null
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" proof run \
    --task "$task_id" --layer quick -- true >/dev/null
  intake_id="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" query sql \
    "SELECT intake_id FROM task WHERE id='$task_id'" | awk 'NR == 3 {print $1}')"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" trace \
    --summary 'Minimal finish fixture' --intake "$intake_id" --agent codex \
    --outcome completed --friction none >/dev/null
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task finish \
    --id "$task_id" --owner codex --session minimal-finish \
    --outcome completed --friction none --json \
    | jq -e '.ok == true and .status == "completed" and (.trace_id | type == "number")' \
      >/dev/null
}

probe_minimal_finish_help() {
  ! "$CLI" task finish --help | sed -n '1p' | grep -q -- '--trace <TRACE>'
}

probe_packet_discovery() {
  HARNESS_REPO_ROOT="$ROOT" "$CLI" memory check --dry-run --json \
    | jq -e '
      .ok == true and
      (.checked | index("docs/stories/CL-71-clp001-terminal-closure/overview.md")) != null and
      (.checked | index("docs/stories/CL-72-command-lifecycle-portable-memory-closure/overview.md")) != null
    ' >/dev/null
}

probe_nested_capsule_discovery() {
  HARNESS_REPO_ROOT="$ROOT" "$CLI" memory check --dry-run --json \
    | jq -e '
      .ok == true and
      (.checked | index("docs/tasks/2026/07/TASK-000022-audited-clp-001-closure-and-found.md")) != null
    ' >/dev/null
}

probe_corrupt_capsule_rejected() {
  local clone capsule output status
  clone="$(new_clone corrupt-capsule)"
  capsule="$clone/docs/tasks/2026/07/TASK-000022-audited-clp-001-closure-and-found.md"
  printf '\ncorruption fixture\n' >>"$capsule"
  if output="$(HARNESS_REPO_ROOT="$clone" "$CLI" memory check --dry-run --json 2>&1)"; then
    status=0
  else
    status=$?
  fi
  test "$status" -ne 0 || test "$(jq -r .ok <<<"$output")" = false
  jq -e '.errors | map(select(contains("TASK-000022"))) | length > 0' \
    <<<"$output" >/dev/null
}

probe_duplicate_packet_rejected() {
  local clone output status
  clone="$(new_clone duplicate-packet)"
  cp "$clone/docs/stories/CL-71-clp001-terminal-closure/overview.md" \
    "$clone/docs/stories/CL-71-duplicate.md"
  if output="$(HARNESS_REPO_ROOT="$clone" "$CLI" memory check --dry-run --json 2>&1)"; then
    status=0
  else
    status=$?
  fi
  test "$status" -ne 0 || test "$(jq -r .ok <<<"$output")" = false
  jq -e '.errors | map(select(contains("duplicate story id CL-71"))) | length > 0' \
    <<<"$output" >/dev/null
}

probe_symlink_escape_rejected() {
  local clone external output status
  clone="$(new_clone symlink-escape)"
  external="$WORK/external-story.md"
  sed 's/US-001/SYMLINK-ESCAPE/g' \
    "$clone/docs/stories/US-001-simple-curl-installer.md" >"$external"
  ln -s "$external" "$clone/docs/stories/SYMLINK-ESCAPE.md"
  if output="$(HARNESS_REPO_ROOT="$clone" "$CLI" memory check --dry-run --json 2>&1)"; then
    status=0
  else
    status=$?
  fi
  test "$status" -ne 0 || test "$(jq -r .ok <<<"$output")" = false
  jq -e '.errors | map(select(test("symlink|escape|unsafe"; "i"))) | length > 0' \
    <<<"$output" >/dev/null
}

probe_case_collision_rejected() {
  local clone output status
  clone="$(new_clone case-collision)"
  sed 's/CL-00/cl-00/g' "$clone/docs/stories/CL-00-freeze-recover-baseline.md" \
    >"$clone/docs/stories/cl-00-case-collision.md"
  if output="$(HARNESS_REPO_ROOT="$clone" "$CLI" memory check --dry-run --json 2>&1)"; then
    status=0
  else
    status=$?
  fi
  test "$status" -ne 0 || test "$(jq -r .ok <<<"$output")" = false
  jq -e '.errors | map(select(test("case-colliding story ids"; "i"))) | length > 0' \
    <<<"$output" >/dev/null
}

probe_unsafe_file_type_rejected() {
  local clone output status
  clone="$(new_clone unsafe-file-type)"
  mkfifo "$clone/docs/tasks/2026/07/unsafe.pipe"
  if output="$(HARNESS_REPO_ROOT="$clone" "$CLI" memory check --dry-run --json 2>&1)"; then
    status=0
  else
    status=$?
  fi
  test "$status" -ne 0 || test "$(jq -r .ok <<<"$output")" = false
  jq -e '.errors | map(select(test("unsafe.*file"; "i"))) | length > 0' \
    <<<"$output" >/dev/null
}

probe_rebuild_projection() {
  local clone output_db output expected expected_schema capsule_rows portable_rows generic_titles
  clone="$(new_clone rebuild-projection)"
  output_db="$clone/clp001-r1-rebuild.db"
  output="$(cd "$clone" && HARNESS_REPO_ROOT="$clone" "$CLI" memory rebuild \
    --dry-run --output clp001-r1-rebuild.db --json)"
  expected=$((
    $(find "$clone/docs/stories" -mindepth 1 -maxdepth 1 -type d | wc -l) +
    $(find "$clone/docs/stories" -mindepth 1 -maxdepth 1 -type f -name '*.md' ! -name README.md | wc -l) +
    $(find "$clone/docs/decisions" -mindepth 1 -maxdepth 1 -type f -name '*.md' ! -name README.md | wc -l) +
    $(find "$clone/docs/tasks" -type f -name '*.md' | wc -l)
  ))
  expected_schema="$(awk '/^version = / { version=$3 } END { print version }' \
    "$clone/_harness/scripts/schema/manifest.toml")"
  jq -e --argjson expected "$expected" --argjson schema "$expected_schema" '
    .ok == true and .temp_schema_version == $schema and
    .artifacts_checked == $expected and .projected_records == $expected and
    .parity.state == "pass" and .parity.schema_version == $schema and
    .parity.projected_count == $expected and .parity.mismatches == []
  ' <<<"$output" >/dev/null
  capsule_rows="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$output_db" "$CLI" query sql \
    "SELECT COUNT(*) FROM artifact_index WHERE artifact_type='capsule'" \
    | awk 'NR == 3 {print $1}')"
  test "${capsule_rows:-0}" -gt 0
  portable_rows="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$output_db" "$CLI" query sql \
    "SELECT COUNT(*) FROM portable_task_summary" | awk 'NR == 3 {print $1}')"
  test "$portable_rows" -eq "$capsule_rows"
  generic_titles="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$output_db" "$CLI" query sql \
    "SELECT COUNT(*) FROM story WHERE id LIKE 'CL-%' AND title='Overview'" \
    | awk 'NR == 3 {print $1}')"
  test "$generic_titles" -eq 0
}

probe_rebuild_is_repeatable_and_dry_run_is_read_only() {
  local clone database before first second after
  clone="$(new_clone rebuild-repeat)"
  database="$clone/harness.db"
  init_database "$clone" "$database"
  before="$(sha256sum "$database" | awk '{print $1}')"
  first="$(cd "$clone" && HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" \
    "$CLI" memory rebuild --dry-run --json | jq -r .logical_digest)"
  second="$(cd "$clone" && HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" \
    "$CLI" memory rebuild --dry-run --json | jq -r .logical_digest)"
  after="$(sha256sum "$database" | awk '{print $1}')"
  test "$first" = "$second"
  test "$before" = "$after"
}

probe_packet_checksum_covers_sorted_components() {
  local clone first_db second_db first_output second_output first_checksum second_checksum
  clone="$(new_clone packet-checksum)"
  first_db="$clone/packet-first.db"
  second_db="$clone/packet-second.db"
  first_output="$(cd "$clone" && HARNESS_REPO_ROOT="$clone" "$CLI" memory rebuild \
    --dry-run --output packet-first.db --json)"
  first_checksum="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$first_db" "$CLI" query sql \
    "SELECT checksum FROM artifact_index WHERE artifact_type='story' AND artifact_id='CL-72'" \
    | awk 'NR == 3 {print $1}')"
  printf '\nAggregate checksum fixture.\n' \
    >>"$clone/docs/stories/CL-72-command-lifecycle-portable-memory-closure/design.md"
  second_output="$(cd "$clone" && HARNESS_REPO_ROOT="$clone" "$CLI" memory rebuild \
    --dry-run --output packet-second.db --json)"
  second_checksum="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$second_db" "$CLI" query sql \
    "SELECT checksum FROM artifact_index WHERE artifact_type='story' AND artifact_id='CL-72'" \
    | awk 'NR == 3 {print $1}')"
  test "$first_checksum" != "$second_checksum"
  test "$(jq -r .logical_digest <<<"$first_output")" != \
    "$(jq -r .logical_digest <<<"$second_output")"
}

probe_apply_is_backup_first_and_preserves_operational_rows() {
  local clone database task_id capsule before output after backup portable_rows
  clone="$(new_clone rebuild-apply)"
  database="$clone/harness.db"
  init_database "$clone" "$database"
  task_id="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type maintenance --summary 'Retained apply fixture' \
    --owner codex --session retained-apply --behavior-bearing no --json | jq -r .task_id)"
  rm "$clone/docs/tasks/2026/07/TASK-000001-canonicalized-main-lineage-operational-db-and.md"
  capsule="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" memory capsule render \
    --id "$task_id" --date 2099-01-01 --lane tiny --outcome completed \
    --summary 'Portable v2 apply fixture' --json | jq -r .path)"
  grep -q '^schema: harness/task-capsule/v2$' "$clone/$capsule"
  before="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" query sql \
    'SELECT COUNT(*) FROM task' | awk 'NR == 3 {print $1}')"
  output="$(cd "$clone" && HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" \
    "$CLI" memory rebuild --apply --json)"
  after="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" query sql \
    'SELECT COUNT(*) FROM task' | awk 'NR == 3 {print $1}')"
  backup="$(jq -r .backup <<<"$output")"
  test "$before" = "$after"
  test -f "$clone/$backup"
  jq -e '
    .ok == true and .mode == "apply" and
    .preserved_operational_state == true and .parity.state == "pass"
  ' <<<"$output" >/dev/null
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task status \
    --id "$task_id" --json | jq -e '.status == "in_progress"' >/dev/null
  portable_rows="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" query sql \
    "SELECT COUNT(*) FROM portable_task_summary WHERE task_id='$task_id' AND capsule_schema='harness/task-capsule/v2'" \
    | awk 'NR == 3 {print $1}')"
  test "$portable_rows" -eq 1
}

probe_named_audit_coverage() {
  local output
  output="$(HARNESS_REPO_ROOT="$ROOT" "$CLI" audit --json)"
  jq -e '
    ((.audit.coverage_checks // .audit.checks) | type == "array") and
    ((.audit.coverage_checks // .audit.checks) |
      map(select((.check_id // .id) == "semantic-memory-parity" and
        (.state == "pass" or .state == "fail" or .state == "unknown" or
         .state == "not_applicable"))) | length == 1)
  ' <<<"$output" >/dev/null
}

probe_absent_parity_is_unknown() {
  local clone database output
  clone="$(new_clone audit-absent)"
  database="$clone/harness.db"
  init_database "$clone" "$database"
  output="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" audit --json)"
  jq -e '
    ((.audit.coverage_checks // .audit.checks) |
      map(select((.check_id // .id) == "semantic-memory-parity" and
        .state == "unknown")) | length == 1)
  ' <<<"$output" >/dev/null
}

probe_failed_parity_is_failed() {
  local clone database task_id output proof_status
  clone="$(new_clone audit-failed)"
  database="$clone/harness.db"
  init_database "$clone" "$database"
  task_id="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type 'maintenance request' --summary 'Failed parity fixture' \
    --owner codex --session audit-failed --behavior-bearing no --json \
    | jq -r .task_id)"
  if HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" proof run \
    --task "$task_id" --layer semantic-memory-parity -- sh -c 'exit 9' \
    >/dev/null 2>&1; then
    proof_status=0
  else
    proof_status=$?
  fi
  test "$proof_status" -ne 0
  output="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" audit --json)"
  jq -e '
    ((.audit.coverage_checks // .audit.checks) |
      map(select((.check_id // .id) == "semantic-memory-parity" and
        .state == "fail")) | length == 1)
  ' <<<"$output" >/dev/null
}

probe_complete_parity_is_current_pass() {
  local clone database task_id output
  clone="$(new_clone audit-pass)"
  database="$clone/harness.db"
  init_database "$clone" "$database"
  task_id="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type 'maintenance request' --summary 'Current parity fixture' \
    --owner codex --session audit-pass --behavior-bearing no --json \
    | jq -r .task_id)"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" proof run \
    --task "$task_id" --layer semantic-memory-parity -- \
    "$CLI" memory rebuild --dry-run --json >/dev/null
  output="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" audit --json)"
  jq -e '
    (.audit.coverage_checks |
      map(select(.check_id == "semantic-memory-parity" and .state == "pass" and
        .freshness.head == true and .freshness.branch == true and
        .freshness.dirty == true and .freshness.output == true and
        .measured_counts.story > 0 and .measured_counts.capsule > 0 and
        .measured_counts.projected > 0 and .measured_counts.schema_version > 0)) |
      length == 1)
  ' <<<"$output" >/dev/null
}

probe_incomplete_parity_is_failed() {
  local clone database task_id output
  clone="$(new_clone audit-incomplete)"
  database="$clone/harness.db"
  init_database "$clone" "$database"
  task_id="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type 'maintenance request' --summary 'Incomplete parity fixture' \
    --owner codex --session audit-incomplete --behavior-bearing no --json \
    | jq -r .task_id)"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" proof run \
    --task "$task_id" --layer semantic-memory-parity -- true >/dev/null
  output="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" audit --json)"
  jq -e '
    (.audit.coverage_checks |
      map(select(.check_id == "semantic-memory-parity" and .state == "fail" and
        .freshness.dirty == true)) | length == 1)
  ' <<<"$output" >/dev/null
}

probe_omitted_indexed_artifact_is_failed() {
  local clone database task_id output
  clone="$(new_clone audit-omitted)"
  database="$clone/harness.db"
  init_database "$clone" "$database"
  task_id="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type 'maintenance request' --summary 'Omitted artifact fixture' \
    --owner codex --session audit-omitted --behavior-bearing no --json \
    | jq -r .task_id)"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" memory rebuild \
    --apply --json >/dev/null
  rm "$clone/docs/tasks/2026/07/TASK-000022-audited-clp-001-closure-and-found.md"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" proof run \
    --task "$task_id" --layer semantic-memory-parity -- \
    "$CLI" memory rebuild --dry-run --json >/dev/null
  output="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" audit --json)"
  jq -e '
    (.audit.coverage_checks |
      map(select(.check_id == "semantic-memory-parity" and .state == "fail" and
        .freshness.dirty == true and (.remediation | length > 0))) | length == 1)
  ' <<<"$output" >/dev/null
}

probe_stale_parity_is_not_pass() {
  local clone database task_id output
  clone="$(new_clone audit-stale)"
  database="$clone/harness.db"
  init_database "$clone" "$database"
  task_id="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" task start \
    --type 'maintenance request' --summary 'Stale parity fixture' \
    --owner codex --session audit-stale --behavior-bearing no --json \
    | jq -r .task_id)"
  HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" proof run \
    --task "$task_id" --layer semantic-memory-parity -- true >/dev/null
  printf 'make semantic parity proof stale\n' >"$clone/clp001-r1-dirty-probe.txt"
  output="$(HARNESS_REPO_ROOT="$clone" HARNESS_DB="$database" "$CLI" audit --json)"
  jq -e '
    ((.audit.coverage_checks // .audit.checks) |
      map(select((.check_id // .id) == "semantic-memory-parity" and
        (.state == "unknown" or .state == "fail") and
        ((if (.freshness | has("dirty")) then .freshness.dirty
          else .dirty_fresh end) == false))) | length == 1)
  ' <<<"$output" >/dev/null
}

COMMAND_PROBES=(
  probe_minimal_start
  probe_start_json_contract
  probe_status_json_contract
  probe_json_error_envelope
  probe_json_parse_error_envelope
  probe_json_preflight_error_envelope
  probe_json_policy_error_envelope
  probe_auto_classification_is_typed
  probe_minimal_finish_executes
  probe_minimal_finish_help
)
MEMORY_PROBES=(
  probe_packet_discovery
  probe_nested_capsule_discovery
  probe_corrupt_capsule_rejected
  probe_duplicate_packet_rejected
  probe_symlink_escape_rejected
  probe_case_collision_rejected
  probe_unsafe_file_type_rejected
  probe_rebuild_projection
  probe_rebuild_is_repeatable_and_dry_run_is_read_only
  probe_packet_checksum_covers_sorted_components
  probe_apply_is_backup_first_and_preserves_operational_rows
)
AUDIT_PROBES=(
  probe_named_audit_coverage
  probe_absent_parity_is_unknown
  probe_failed_parity_is_failed
  probe_complete_parity_is_current_pass
  probe_incomplete_parity_is_failed
  probe_omitted_indexed_artifact_is_failed
  probe_stale_parity_is_not_pass
)

selected_probes() {
  case "$MODE" in
    command) printf '%s\n' "${COMMAND_PROBES[@]}" ;;
    memory) printf '%s\n' "${MEMORY_PROBES[@]}" ;;
    audit) printf '%s\n' "${AUDIT_PROBES[@]}" ;;
    all|baseline)
      printf '%s\n' "${COMMAND_PROBES[@]}" "${MEMORY_PROBES[@]}" \
        "${AUDIT_PROBES[@]}"
      ;;
  esac
}

if test "$MODE" = baseline; then
  reproduced=0
  set +e
  while IFS= read -r probe; do
    "$probe" >"$WORK/$probe.out" 2>&1
    probe_status=$?
    if test "$probe_status" -eq 0; then
      printf 'baseline unexpectedly passed: %s\n' "$probe" >&2
      exit 1
    fi
    reproduced=$((reproduced + 1))
    printf 'reproduced gap: %s\n' "$probe"
  done < <(selected_probes)
  set -e
  printf 'CLP-001-R1 baseline: %s intended gaps reproduced\n' "$reproduced"
  exit 0
fi

while IFS= read -r probe; do
  "$probe"
  printf 'passed contract: %s\n' "$probe"
done < <(selected_probes)
