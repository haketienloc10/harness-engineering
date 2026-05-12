#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNS_DIR="$ROOT_DIR/.harness/runs"
LEGACY_EPICS_DIR="$ROOT_DIR/.harness/epics"

echo "== Harness Epic Containers =="

if find "$RUNS_DIR" -maxdepth 1 -type d -name 'EPIC-*' -print -quit 2>/dev/null | grep -q .; then
  find "$RUNS_DIR" -maxdepth 1 -type d -name 'EPIC-*' -printf '%f\n' 2>/dev/null | sort
else
  echo "No Harness epics found. Create one with:"
  echo "  bash .harness/scripts/new-epic.sh \"long task name\""
fi

if [ -d "$LEGACY_EPICS_DIR" ]; then
  echo
  echo "== Legacy .harness/epics =="
  find "$LEGACY_EPICS_DIR" -maxdepth 1 -type d -name 'EPIC-*' -printf '%f\n' 2>/dev/null | sort || true
fi
