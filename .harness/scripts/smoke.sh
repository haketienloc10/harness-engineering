#!/usr/bin/env bash
set -euo pipefail

echo "== Harness Smoke Check =="

APP_URL="${APP_URL:-http://localhost:3000}"

echo "Checking $APP_URL"

curl -fsS "$APP_URL" > /tmp/harness-smoke-output.txt

echo "Smoke check passed: $APP_URL"
echo "Output saved to: /tmp/harness-smoke-output.txt"
