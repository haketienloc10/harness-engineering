#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$(mktemp -d)"
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
  PATH="$MOCK_BIN:$PATH" HARNESS_TEST_ARCHIVE="$ARCHIVE" HARNESS_LITE_TARGET_DIR="$TARGET" HARNESS_LITE_OWNER="test" HARNESS_LITE_REPO="harness" HARNESS_LITE_REF="fixture" bash "$ROOT/install.sh"
}

run_install >"$WORK/first.log"

test -f "$TARGET/.harness-id"
test -f "$TARGET/AGENTS.md"
test -f "$TARGET/_harness/.harness-manifest"
test ! -e "$TARGET/_harness/HARNESS.md"
test ! -e "$TARGET/_harness/FEATURE_INTAKE.md"
test ! -e "$TARGET/_harness/CONTEXT_RULES.md"
test ! -e "$TARGET/_harness/TRACE_SPEC.md"
test ! -e "$TARGET/_harness/TEST_MATRIX.md"
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

"$TARGET/_harness/bin/harness-cli" workflow validate --json | grep -q '"mode":"shadow"'
"$TARGET/_harness/bin/harness-cli" workflow parity --json | grep -q '"code":"WORKFLOW_PARITY_OK"'
"$TARGET/_harness/bin/harness-cli" workflow commands >"$WORK/installed-commands.txt"
grep -v '^#' "$TARGET/_harness/command-manifest.txt" | sed '/^[[:space:]]*$/d' >"$WORK/tracked-commands.txt"
cmp "$WORK/tracked-commands.txt" "$WORK/installed-commands.txt"

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

printf 'installer state safety: ok\n'
