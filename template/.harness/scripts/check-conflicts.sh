#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNS_DIR="$ROOT_DIR/.harness/runs"

TARGET_RUN="${1:-}"

if [ -z "$TARGET_RUN" ]; then
  echo "Usage: bash .harness/scripts/check-conflicts.sh RUN-YYYYMMDD-NNN-slug"
  echo "   or: bash .harness/scripts/check-conflicts.sh EPIC-YYYYMMDD-NNN-slug/runs/RUN-NNN-slug"
  exit 2
fi

if [ -d "$TARGET_RUN" ]; then
  TARGET_DIR="$TARGET_RUN"
else
  TARGET_DIR="$RUNS_DIR/$TARGET_RUN"
fi
[ -d "$TARGET_DIR" ] && TARGET_DIR="$(cd "$TARGET_DIR" && pwd -P)"
TARGET_CONTRACT="$TARGET_DIR/02-implementation-contract.md"

if [ ! -f "$TARGET_CONTRACT" ]; then
  echo "Target contract not found: $TARGET_CONTRACT"
  exit 2
fi

extract_files() {
  local file="$1"
  grep -E '^\|[[:space:]]*([^|]+)[[:space:]]*\|' "$file" 2>/dev/null \
    | grep -vE 'Area/File|---' \
    | awk -F'|' '{gsub(/^[ \t]+|[ \t]+$/, "", $2); print $2}' \
    | grep -E '[/\.]' \
    | sort -u || true
}

target_files="$(extract_files "$TARGET_CONTRACT")"

if [ -z "$target_files" ]; then
  echo "No expected files detected in target contract."
  echo "Tip: fill 'Files / Areas Expected to Change' before conflict check."
  exit 0
fi

echo "== Target run =="
echo "$TARGET_RUN"
echo
echo "== Target expected files/areas =="
echo "$target_files"
echo

found=0

while IFS= read -r run_dir; do
  [ -d "$run_dir" ] || continue

  run_id="$(basename "$run_dir")"
  [ "$run_dir" = "$TARGET_DIR" ] && continue

  yaml="$run_dir/run.yaml"
  contract="$run_dir/02-implementation-contract.md"

  status="unknown"
  if [ -f "$yaml" ]; then
    status="$(grep -E '^state:' "$yaml" | head -1 | cut -d: -f2- | xargs || true)"
  fi

  case "$status" in
    COMPLETED|CANCELLED)
      continue
      ;;
  esac

  [ -f "$contract" ] || continue

  other_files="$(extract_files "$contract")"
  [ -z "$other_files" ] && continue

  overlap="$(comm -12 <(printf "%s\n" "$target_files" | sort -u) <(printf "%s\n" "$other_files" | sort -u) || true)"

  if [ -n "$overlap" ]; then
    found=1
    echo "Potential conflict with: $run_id (state: $status)"
    echo "$overlap" | sed 's/^/  - /'
    echo
  fi
done < <(
  {
    find "$RUNS_DIR" -maxdepth 1 -type d -name 'RUN-*' 2>/dev/null
    find "$RUNS_DIR" -mindepth 3 -maxdepth 3 -type d -path "$RUNS_DIR/EPIC-*/runs/RUN-*" 2>/dev/null
  } | sort
)

if [ "$found" -eq 0 ]; then
  echo "No overlapping expected files detected against active runs."
else
  echo "Conflict check found potential overlaps. Review before implementation."
  exit 1
fi
