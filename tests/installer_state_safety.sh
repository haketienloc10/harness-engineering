#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$(mktemp -d /dev/shm/harness-installer.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

ARCHIVE="$WORK/source.tar.gz"
tar --transform='s#^\.#harness-fixture#' --exclude=.git --exclude=target --exclude=harness.db --exclude=harness.db-wal --exclude=harness.db-shm -czf "$ARCHIVE" -C "$ROOT" .

MOCK_BIN="$WORK/mock-bin"
mkdir -p "$MOCK_BIN"
cat >"$MOCK_BIN/curl" <<'EOF'
#!/usr/bin/env bash
set -Eeuo pipefail
cp "$HARNESS_TEST_ARCHIVE" "${!#}"
EOF
chmod 755 "$MOCK_BIN/curl"

TARGET="$WORK/target"
mkdir -p "$TARGET/docs/product"
printf 'existing product contract\n' >"$TARGET/docs/product/existing.md"
printf '# user ignore\ncustom.cache\n' >"$TARGET/.gitignore"
cp "$ROOT/harness.db" "$TARGET/harness.db"
DB_HASH_BEFORE="$(sha256sum "$TARGET/harness.db" | awk '{print $1}')"

run_install() {
  local target="${1:-$TARGET}"
  PATH="$MOCK_BIN:$PATH" HARNESS_TEST_ARCHIVE="$ARCHIVE" HARNESS_LITE_TARGET_DIR="$target" HARNESS_LITE_OWNER="test" HARNESS_LITE_REPO="harness" HARNESS_LITE_REF="fixture" bash "$ROOT/install.sh"
}

run_install >"$WORK/first.log"

# The positional target is the shortest local-install form. A named source ref
# overrides the legacy ref variable and must be persisted in the manifest.
OVERRIDE_TARGET="$WORK/override-target"
mkdir -p "$OVERRIDE_TARGET"
PATH="$MOCK_BIN:$PATH" HARNESS_TEST_ARCHIVE="$ARCHIVE" \
  HARNESS_LITE_TARGET_DIR="$WORK/ignored-target" \
  HARNESS_LITE_OWNER="test" HARNESS_LITE_REPO="harness" \
  HARNESS_LITE_REF="fixture" HARNESS_LITE_SOURCE_REF="feature-rework" \
  bash "$ROOT/install.sh" "$OVERRIDE_TARGET" >"$WORK/override.log"
test -f "$OVERRIDE_TARGET/_harness/.harness-manifest"
test ! -e "$WORK/ignored-target"
grep -qx 'ref = feature-rework' "$OVERRIDE_TARGET/_harness/.harness-manifest"

test -f "$TARGET/.harness-id"
test -f "$TARGET/AGENTS.md"
test -f "$TARGET/_harness/.harness-manifest"
test -x "$TARGET/_harness/bin/harness-cli"
test -f "$TARGET/_harness/workflow.toml"
test -f "$TARGET/_harness/command-manifest.txt"
test -f "$TARGET/_harness/templates/story.md"
test -f "$TARGET/_harness/scripts/schema/manifest.toml"
test ! -e "$TARGET/_harness/HARNESS.md"
test ! -e "$TARGET/_harness/FEATURE_INTAKE.md"
test ! -e "$TARGET/_harness/CONTEXT_RULES.md"
test ! -e "$TARGET/_harness/TRACE_SPEC.md"
test ! -e "$TARGET/_harness/TEST_MATRIX.md"
test ! -e "$TARGET/_harness/docs"
test ! -e "$TARGET/.agents"
test "$(cat "$TARGET/docs/product/existing.md")" = "existing product contract"
test "$(sha256sum "$TARGET/harness.db" | awk '{print $1}')" = "$DB_HASH_BEFORE"
grep -qx 'custom.cache' "$TARGET/.gitignore"
test "$(grep -c '^# HARNESS:BEGIN local state$' "$TARGET/.gitignore")" -eq 1
FIRST_ID="$(cat "$TARGET/.harness-id")"

awk '
  $0 == "<!-- HARNESS:SHARED:BEGIN -->" { capture=1; next }
  $0 == "<!-- HARNESS:SHARED:END -->" { capture=0; next }
  capture { print }
' "$TARGET/AGENTS.md" >"$WORK/installed-shared-agents.md"
cmp "$ROOT/AGENTS.md" "$WORK/installed-shared-agents.md"
grep -q '^# Harness đã cài đặt$' "$TARGET/AGENTS.md"
grep -q 'runtime cho agent' "$TARGET/AGENTS.md"

"$TARGET/_harness/bin/harness-cli" workflow validate --json | grep -q '"mode":"shadow"'
"$TARGET/_harness/bin/harness-cli" workflow parity --json | grep -q '"code":"WORKFLOW_PARITY_OK"'
"$TARGET/_harness/bin/harness-cli" workflow commands >"$WORK/installed-commands.txt"
grep -v '^#' "$TARGET/_harness/command-manifest.txt" | sed '/^[[:space:]]*$/d' >"$WORK/tracked-commands.txt"
cmp "$WORK/tracked-commands.txt" "$WORK/installed-commands.txt"

# A completely fresh target can start/status a
# tiny task without any pre-created docs/ directories. Product records remain
# lazy-created by the workflow that needs them.
FRESH_TARGET="$WORK/fresh-target"
mkdir -p "$FRESH_TARGET"
git init -q "$FRESH_TARGET"
run_install "$FRESH_TARGET" >"$WORK/fresh.log"
FRESH_CLI="$FRESH_TARGET/_harness/bin/harness-cli"
START_JSON="$(cd "$FRESH_TARGET" && ./_harness/bin/harness-cli task start --type 'maintenance request' \
  --summary 'installer fresh-target smoke' --lane tiny \
  --lane-reason 'installer smoke test' --owner smoke --session fresh-target \
  --behavior-bearing no --json)"
TASK_ID="$(printf '%s\n' "$START_JSON" | sed -n 's/.*"task_id":"\([^"]*\)".*/\1/p')"
test -n "$TASK_ID"
(cd "$FRESH_TARGET" && ./_harness/bin/harness-cli task status --id "$TASK_ID" --json) | grep -q '"status":"in_progress"'
test ! -e "$FRESH_TARGET/docs"

run_install >"$WORK/second.log"

test "$(cat "$TARGET/.harness-id")" = "$FIRST_ID"
test "$(grep -c '^# HARNESS:BEGIN local state$' "$TARGET/.gitignore")" -eq 1
test "$(sha256sum "$TARGET/harness.db" | awk '{print $1}')" = "$DB_HASH_BEFORE"
grep -q 'harness.db không bị thay đổi' "$WORK/second.log"

cat >"$MOCK_BIN/uname" <<'EOF'
#!/usr/bin/env bash
if [ "${1:-}" = "-s" ]; then printf 'Darwin\n'; else printf 'arm64\n'; fi
EOF
chmod 755 "$MOCK_BIN/uname"
if PATH="$MOCK_BIN:$PATH" HARNESS_TEST_ARCHIVE="$ARCHIVE" HARNESS_LITE_TARGET_DIR="$TARGET" bash "$ROOT/install.sh" >"$WORK/platform.log" 2>&1; then
  echo "unsupported platform unexpectedly installed" >&2
  exit 1
fi
grep -q 'Không có harness-cli binary' "$WORK/platform.log"
rm "$MOCK_BIN/uname"

MISSING_SOURCE="$WORK/missing-source"
mkdir -p "$MISSING_SOURCE"
tar -xzf "$ARCHIVE" -C "$MISSING_SOURCE"
rm "$MISSING_SOURCE/harness-fixture/_harness/bin/harness-cli"
MISSING_ARCHIVE="$WORK/missing-cli.tar.gz"
tar -czf "$MISSING_ARCHIVE" -C "$MISSING_SOURCE" harness-fixture
MISSING_TARGET="$WORK/missing-target"
mkdir -p "$MISSING_TARGET"
printf 'preserve before failed install\n' >"$MISSING_TARGET/user-file.txt"
if PATH="$MOCK_BIN:$PATH" HARNESS_TEST_ARCHIVE="$MISSING_ARCHIVE" \
  HARNESS_LITE_TARGET_DIR="$MISSING_TARGET" HARNESS_LITE_OWNER="test" \
  HARNESS_LITE_REPO="harness" HARNESS_LITE_REF="fixture" \
  bash "$ROOT/install.sh" >"$WORK/missing-cli.log" 2>&1; then
  echo "archive without harness-cli unexpectedly installed" >&2
  exit 1
fi
grep -q 'Thiếu executable _harness/bin/harness-cli' "$WORK/missing-cli.log"
test "$(cat "$MISSING_TARGET/user-file.txt")" = "preserve before failed install"
test ! -e "$MISSING_TARGET/AGENTS.md"
test ! -e "$MISSING_TARGET/_harness"

printf 'installer state safety: ok\n'
