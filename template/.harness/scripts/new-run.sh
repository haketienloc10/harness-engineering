#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS_DIR="$ROOT_DIR/.harness"
RUNS_DIR="$HARNESS_DIR/runs"
TEMPLATES_DIR="$HARNESS_DIR/templates"
RUN_INDEX="$RUNS_DIR/RUN_INDEX.md"

raw_slug="${1:-untitled}"

slug="$(printf "%s" "$raw_slug" \
  | tr '[:upper:]' '[:lower:]' \
  | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//; s/-+/-/g')"

if [ -z "$slug" ]; then
  slug="untitled"
fi

TODAY="$(date +%Y%m%d)"
NOW="$(date -Iseconds)"

mkdir -p "$RUNS_DIR"

N=1
while true; do
  RUN_ID="RUN-${TODAY}-$(printf "%03d" "$N")-${slug}"
  RUN_DIR="$RUNS_DIR/$RUN_ID"
  if [ ! -d "$RUN_DIR" ]; then
    break
  fi
  N=$((N + 1))
done

mkdir -p "$RUN_DIR"

cp "$TEMPLATES_DIR/run.yaml.template" "$RUN_DIR/run.yaml"
cp "$TEMPLATES_DIR/00-input.template.md" "$RUN_DIR/00-input.md"
cp "$TEMPLATES_DIR/01-planner-brief.template.md" "$RUN_DIR/01-planner-brief.md"
cp "$TEMPLATES_DIR/02-implementation-contract.template.md" "$RUN_DIR/02-implementation-contract.md"
cp "$TEMPLATES_DIR/03-evaluator-contract-review.template.md" "$RUN_DIR/03-evaluator-contract-review.md"
cp "$TEMPLATES_DIR/04-generator-worklog.template.md" "$RUN_DIR/04-generator-worklog.md"
cp "$TEMPLATES_DIR/05-evaluator-report.template.md" "$RUN_DIR/05-evaluator-report.md"
cp "$TEMPLATES_DIR/06-fix-report.template.md" "$RUN_DIR/06-fix-report.md"
cp "$TEMPLATES_DIR/07-final-summary.template.md" "$RUN_DIR/07-final-summary.md"

replace_placeholders() {
  local file="$1"
  if command -v perl >/dev/null 2>&1; then
    perl -pi -e "s/<RUN-ID>/$RUN_ID/g; s/<TASK-SLUG>/$slug/g; s/<CREATED-AT>/$NOW/g; s/<UPDATED-AT>/$NOW/g" "$file"
  else
    sed -i "s/<RUN-ID>/$RUN_ID/g; s/<TASK-SLUG>/$slug/g; s/<CREATED-AT>/$NOW/g; s/<UPDATED-AT>/$NOW/g" "$file"
  fi
}

for f in "$RUN_DIR"/*; do
  [ -f "$f" ] && replace_placeholders "$f"
done

if [ ! -f "$RUN_INDEX" ]; then
  cat > "$RUN_INDEX" <<'EOF'
# Harness Run Index

| Run ID | Task | Status | Branch | Worktree | Owner | Started At | Last Updated |
|---|---|---|---|---|---|---|---|
EOF
fi

printf "| %s | %s | created |  |  | agent | %s | %s |\n" "$RUN_ID" "$slug" "$NOW" "$NOW" >> "$RUN_INDEX"

echo "Created Harness run:"
echo "$RUN_DIR"
