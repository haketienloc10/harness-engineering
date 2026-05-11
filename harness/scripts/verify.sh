#!/usr/bin/env bash
set -euo pipefail

echo "== Harness Verify =="

if [ -f "package.json" ]; then
  echo "== Node project detected =="

  if npm run | grep -qE "^[[:space:]]+lint"; then
    npm run lint
  fi

  if npm run | grep -qE "^[[:space:]]+typecheck"; then
    npm run typecheck
  fi

  if npm run | grep -qE "^[[:space:]]+test"; then
    npm test
  fi

  if npm run | grep -qE "^[[:space:]]+build"; then
    npm run build
  fi
fi

if [ -f "pom.xml" ]; then
  echo "== Maven project detected =="
  mvn test
fi

if [ -f "build.gradle" ] || [ -f "build.gradle.kts" ]; then
  echo "== Gradle project detected =="
  if [ -x "./gradlew" ]; then
    ./gradlew test
  else
    gradle test
  fi
fi

echo "== Verify completed =="
