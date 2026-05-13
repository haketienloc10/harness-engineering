#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cat >&2 <<'EOF'
Warning: mark-subagent-runtime.sh is deprecated.
Use set-runtime-capability.sh. This records a manual assertion, not a runtime proof.
EOF

exec bash "$SCRIPT_DIR/set-runtime-capability.sh" "$@"
