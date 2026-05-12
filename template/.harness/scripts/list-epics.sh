#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
EPIC_INDEX="$ROOT_DIR/.harness/epics/EPIC_INDEX.md"

if [ -f "$EPIC_INDEX" ]; then
  cat "$EPIC_INDEX"
else
  echo "No Harness epics found. Create one with:"
  echo "  bash .harness/scripts/new-epic.sh \"long task name\""
fi
