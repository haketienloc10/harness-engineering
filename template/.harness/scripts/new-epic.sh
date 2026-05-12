#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS_DIR="$ROOT_DIR/.harness"
EPICS_DIR="$HARNESS_DIR/epics"
TEMPLATES_DIR="$HARNESS_DIR/templates"
EPIC_INDEX="$EPICS_DIR/EPIC_INDEX.md"

usage() {
  cat <<'EOF'
Usage:
  bash .harness/scripts/new-epic.sh "long task name"

Create a new Harness Epic for a long-running task.
EOF
}

die() {
  echo "Error: $*" >&2
  exit 1
}

slugify() {
  local raw="$1"
  local slug

  slug="$(printf "%s" "$raw" \
    | tr '[:upper:]' '[:lower:]' \
    | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//; s/-+/-/g')"

  if [ -z "$slug" ]; then
    slug="untitled"
  fi

  printf "%s" "$slug"
}

write_epic_index_if_missing() {
  if [ ! -f "$EPIC_INDEX" ]; then
    cat > "$EPIC_INDEX" <<'EOF'
# Harness Epic Index

| Epic ID | Task | Status | Owner | Started At | Last Updated | Active Run | Notes |
|---|---|---|---|---|---|---|---|
EOF
  fi
}

replace_placeholders() {
  local file="$1"

  if command -v perl >/dev/null 2>&1; then
    perl -pi -e "s/<EPIC-ID>/$EPIC_ID/g; s/<TASK-SLUG>/$slug/g; s/<CREATED-AT>/$NOW/g; s/<UPDATED-AT>/$NOW/g" "$file"
  else
    sed -i "s/<EPIC-ID>/$EPIC_ID/g; s/<TASK-SLUG>/$slug/g; s/<CREATED-AT>/$NOW/g; s/<UPDATED-AT>/$NOW/g" "$file"
  fi
}

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi

raw_slug="${*:-untitled}"
slug="$(slugify "$raw_slug")"

[ -d "$TEMPLATES_DIR" ] || die "Templates directory not found: $TEMPLATES_DIR"

TODAY="$(date +%Y%m%d)"
NOW="$(date -Iseconds)"

mkdir -p "$EPICS_DIR"
write_epic_index_if_missing

N=1
while true; do
  EPIC_ID="EPIC-${TODAY}-$(printf "%03d" "$N")-${slug}"
  EPIC_DIR="$EPICS_DIR/$EPIC_ID"
  if [ ! -d "$EPIC_DIR" ]; then
    break
  fi
  N=$((N + 1))
done

mkdir -p "$EPIC_DIR"

cp "$TEMPLATES_DIR/epic.yaml.template" "$EPIC_DIR/epic.yaml"
cp "$TEMPLATES_DIR/00-epic-overview.template.md" "$EPIC_DIR/00-epic-overview.md"
cp "$TEMPLATES_DIR/01-epic-roadmap.template.md" "$EPIC_DIR/01-roadmap.md"
cp "$TEMPLATES_DIR/02-epic-acceptance-matrix.template.md" "$EPIC_DIR/02-acceptance-matrix.md"
cp "$TEMPLATES_DIR/03-epic-decision-log.template.md" "$EPIC_DIR/03-decision-log.md"
cp "$TEMPLATES_DIR/04-epic-run-index.template.md" "$EPIC_DIR/04-run-index.md"

for f in "$EPIC_DIR"/*; do
  [ -f "$f" ] && replace_placeholders "$f"
done

printf "| %s | %s | active | agent | %s | %s |  |  |\n" "$EPIC_ID" "$slug" "$NOW" "$NOW" >> "$EPIC_INDEX"

echo "Created Harness epic:"
echo ".harness/epics/$EPIC_ID/"
