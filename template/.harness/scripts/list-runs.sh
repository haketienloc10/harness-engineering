#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNS_DIR="$ROOT_DIR/.harness/runs"

echo "== Harness Runs =="

if [ -f "$RUNS_DIR/RUN_INDEX.md" ]; then
  cat "$RUNS_DIR/RUN_INDEX.md"
else
  echo "No RUN_INDEX.md found."
fi

echo
echo "== Run folders =="
find "$RUNS_DIR" -maxdepth 1 -type d -name 'RUN-*' -printf '%f\n' 2>/dev/null | sort || true
