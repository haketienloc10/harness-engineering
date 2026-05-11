#!/usr/bin/env bash
set -euo pipefail

HARNESS_REPO="${HARNESS_REPO:-https://github.com/haketienloc10/harness-engineering}"
HARNESS_REF="${HARNESS_REF:-main}"
HARNESS_TARBALL_URL="${HARNESS_TARBALL_URL:-}"
BOOTSTRAP_TMPDIR=""

usage() {
  cat <<'EOF'
Usage:
  curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/scripts/install-harness.sh?$(date +%s)" \
    | bash -s -- [installer options]

This bootstrap script downloads the Harness template tarball, then forwards all
arguments to harness/scripts/install.sh.

Common installer options:
  --target PATH
  --agents-mode ask|merge|preserve|replace|backup
  --yes
  --force

Bootstrap environment overrides:
  HARNESS_REPO          Default: https://github.com/haketienloc10/harness-engineering
  HARNESS_REF           Default: main
  HARNESS_TARBALL_URL   Override the tarball URL completely
EOF
}

die() {
  echo "Error: $*" >&2
  exit 1
}

info() {
  echo "==> $*"
}

tarball_url() {
  if [ -n "$HARNESS_TARBALL_URL" ]; then
    printf "%s" "$HARNESS_TARBALL_URL"
    return
  fi

  printf "%s/archive/refs/heads/%s.tar.gz" "$HARNESS_REPO" "$HARNESS_REF"
}

main() {
  if [ "${1:-}" = "--bootstrap-help" ]; then
    usage
    exit 0
  fi

  command -v curl >/dev/null 2>&1 || die "curl is required"
  command -v tar >/dev/null 2>&1 || die "tar is required"
  command -v mktemp >/dev/null 2>&1 || die "mktemp is required"

  local archive url installer
  BOOTSTRAP_TMPDIR="$(mktemp -d)"
  archive="$BOOTSTRAP_TMPDIR/harness.tar.gz"
  url="$(tarball_url)"

  cleanup() {
    if [ -n "$BOOTSTRAP_TMPDIR" ]; then
      rm -rf "$BOOTSTRAP_TMPDIR"
    fi
  }
  trap cleanup EXIT INT TERM

  info "Downloading Harness from $url"
  curl -fsSL "$url" -o "$archive"

  info "Extracting Harness template"
  tar -xzf "$archive" -C "$BOOTSTRAP_TMPDIR"

  installer="$(find "$BOOTSTRAP_TMPDIR" -path "*/harness/scripts/install.sh" -type f | head -n 1 || true)"
  [ -n "$installer" ] || die "Could not find harness/scripts/install.sh in downloaded archive"

  info "Running Harness installer"
  bash "$installer" "$@"
}

main "$@"
