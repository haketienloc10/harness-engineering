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
echo "== Root run folders =="
find "$RUNS_DIR" -maxdepth 1 -type d -name 'RUN-*' -printf '%f\n' 2>/dev/null | sort || true

echo
echo "== Epic containers =="
find "$RUNS_DIR" -maxdepth 1 -type d -name 'EPIC-*' -printf '%f\n' 2>/dev/null | sort || true

echo
echo "== Child run folders =="
find "$RUNS_DIR" -mindepth 3 -maxdepth 3 -type d -path "$RUNS_DIR/EPIC-*/runs/RUN-*" -printf '%P\n' 2>/dev/null | sort || true
