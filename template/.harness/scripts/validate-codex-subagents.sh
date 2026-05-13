#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

die() {
  echo "ERROR: $*" >&2
  exit 1
}

require_file() {
  local file="$1"
  [ -f "$ROOT_DIR/$file" ] || die "Missing required file: $file"
}

require_file ".codex/config.toml"
require_file ".codex/agents/harness-planner.toml"
require_file ".codex/agents/harness-contract-reviewer.toml"
require_file ".codex/agents/harness-generator.toml"
require_file ".codex/agents/harness-evaluator.toml"

grep -qE '^name[[:space:]]*=[[:space:]]*"harness_planner"[[:space:]]*$' "$ROOT_DIR/.codex/agents/harness-planner.toml" || die "harness-planner.toml must define name = \"harness_planner\""
grep -qE '^name[[:space:]]*=[[:space:]]*"harness_contract_reviewer"[[:space:]]*$' "$ROOT_DIR/.codex/agents/harness-contract-reviewer.toml" || die "harness-contract-reviewer.toml must define name = \"harness_contract_reviewer\""
grep -qE '^name[[:space:]]*=[[:space:]]*"harness_generator"[[:space:]]*$' "$ROOT_DIR/.codex/agents/harness-generator.toml" || die "harness-generator.toml must define name = \"harness_generator\""
grep -qE '^name[[:space:]]*=[[:space:]]*"harness_evaluator"[[:space:]]*$' "$ROOT_DIR/.codex/agents/harness-evaluator.toml" || die "harness-evaluator.toml must define name = \"harness_evaluator\""

if grep -RniE 'coordinator[^[:cntrl:]]*(may|can|allowed to|is allowed to)[[:space:]]+(create|write|invent|generate)[^[:cntrl:]]*(free[ -]form)[^[:cntrl:]]*(role[ -]prompt|prompt)' \
  "$ROOT_DIR/AGENTS.md" "$ROOT_DIR/.harness/guides" "$ROOT_DIR/.harness/workflows" "$ROOT_DIR/.harness/README.md" >/dev/null 2>&1; then
  die "Lifecycle docs allow coordinator-created free-form role prompts"
fi

if grep -RniE '\.harness/subagents/[^[:cntrl:]]*(canonical|source|spawn|instantiate|role prompt)' \
  "$ROOT_DIR/AGENTS.md" "$ROOT_DIR/.harness/guides" "$ROOT_DIR/.harness/workflows" "$ROOT_DIR/.harness/README.md" >/dev/null 2>&1; then
  die "Lifecycle docs still treat .harness/subagents/*.md as canonical role prompt source"
fi

if grep -RniE '(fallback_allowed:[[:space:]]*true|degraded[[:space:]-]+single[[:space:]-]+session[[:space:]-]+fallback[^[:cntrl:]]*(allowed|true)|single[[:space:]-]+session[^[:cntrl:]]*fallback[^[:cntrl:]]*(allowed|true))' \
  "$ROOT_DIR/AGENTS.md" "$ROOT_DIR/.harness/guides" "$ROOT_DIR/.harness/workflows" "$ROOT_DIR/.harness/README.md" "$ROOT_DIR/.harness/templates" >/dev/null 2>&1; then
  die "Lifecycle docs or templates allow fallback single-session/degraded execution"
fi

echo "OK codex_project_subagents"
