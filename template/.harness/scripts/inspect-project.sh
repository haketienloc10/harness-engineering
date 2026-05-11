#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PROJECT_DIR="$ROOT_DIR/.harness/project"

START_MARKER="<!-- HARNESS:GENERATED:START -->"
END_MARKER="<!-- HARNESS:GENERATED:END -->"

has_path() {
  [ -e "$ROOT_DIR/$1" ]
}

has_dir() {
  [ -d "$ROOT_DIR/$1" ]
}

has_glob() {
  local pattern="$1"
  local match
  for match in "$ROOT_DIR"/$pattern; do
    [ -e "$match" ] && return 0
  done
  return 1
}

print_fact() {
  local label="$1"
  local path="$2"

  if [ -e "$ROOT_DIR/$path" ]; then
    printf -- "- %s: \`%s\`\n" "$label" "$path"
  fi
}

print_glob_fact() {
  local label="$1"
  local pattern="$2"
  local match rel found

  found=0
  for match in "$ROOT_DIR"/$pattern; do
    [ -e "$match" ] || continue
    rel="${match#$ROOT_DIR/}"
    printf -- "- %s: \`%s\`\n" "$label" "$rel"
    found=1
  done

  [ "$found" -eq 1 ]
}

package_has_script() {
  local script="$1"
  [ -f "$ROOT_DIR/package.json" ] || return 1
  grep -Eq "\"$script\"[[:space:]]*:" "$ROOT_DIR/package.json"
}

update_generated_section() {
  local file="$1"
  local title="$2"
  local generated="$3"
  local tmp

  mkdir -p "$(dirname "$file")"

  if [ ! -f "$file" ]; then
    {
      printf "# %s\n\n" "$title"
      printf "File này thuộc target repository sau khi install.\n\n"
      printf "Target repository có quyền chỉnh sửa file này. Generated discovery chỉ là observed facts, không phải absolute truth.\n\n"
      printf "## Generated Discovery\n\n"
      printf "%s\n" "$START_MARKER"
      cat "$generated"
      printf "%s\n\n" "$END_MARKER"
      printf "## Manual Notes\n\n"
      printf -- "- Thêm ghi chú local ở đây.\n"
    } > "$file"
    return
  fi

  if grep -Fq "$START_MARKER" "$file" && grep -Fq "$END_MARKER" "$file"; then
    tmp="$(mktemp)"
    awk -v start="$START_MARKER" -v end="$END_MARKER" -v generated="$generated" '
      $0 == start {
        print
        while ((getline line < generated) > 0) print line
        in_generated = 1
        next
      }
      $0 == end {
        in_generated = 0
        print
        next
      }
      !in_generated { print }
    ' "$file" > "$tmp"
    mv "$tmp" "$file"
  else
    tmp="$(mktemp)"
    {
      cat "$file"
      printf "\n## Generated Discovery\n\n"
      printf "%s\n" "$START_MARKER"
      cat "$generated"
      printf "%s\n" "$END_MARKER"
    } > "$tmp"
    mv "$tmp" "$file"
  fi
}

write_stack_profile() {
  local out="$1"

  {
    printf "Generated at: \`%s\`\n\n" "$(date -Iseconds)"
    printf "Discovery output records observed files and directories only.\n\n"

    printf "## Node\n\n"
    found=0
    for item in package.json package-lock.json pnpm-lock.yaml yarn.lock tsconfig.json; do
      if has_path "$item"; then print_fact "$item" "$item"; found=1; fi
    done
    if print_glob_fact "Vite config" "vite.config.*"; then found=1; fi
    if print_glob_fact "Next config" "next.config.*"; then found=1; fi
    [ "$found" -eq 1 ] || printf -- "- No Node indicators observed.\n"

    printf "\n## Java\n\n"
    found=0
    for item in pom.xml build.gradle build.gradle.kts gradlew src/main/java src/test/java; do
      if has_path "$item"; then print_fact "$item" "$item"; found=1; fi
    done
    [ "$found" -eq 1 ] || printf -- "- No Java indicators observed.\n"

    printf "\n## Python\n\n"
    found=0
    for item in pyproject.toml requirements.txt setup.py; do
      if has_path "$item"; then print_fact "$item" "$item"; found=1; fi
    done
    [ "$found" -eq 1 ] || printf -- "- No Python indicators observed.\n"

    printf "\n## Runtime / Infra\n\n"
    found=0
    for item in Dockerfile docker-compose.yml .github/workflows; do
      if has_path "$item"; then print_fact "$item" "$item"; found=1; fi
    done
    [ "$found" -eq 1 ] || printf -- "- No runtime/infra indicators observed.\n"
  } > "$out"
}

write_validation_profile() {
  local out="$1"

  {
    printf "Generated at: \`%s\`\n\n" "$(date -Iseconds)"
    printf "Discovery output records candidate validation commands from observed files only.\n\n"

    printf "## Candidate Commands\n\n"
    found=0

    if has_path "package.json"; then
      for script in lint typecheck test build; do
        if package_has_script "$script"; then
          printf -- "- \`npm run %s\` observed in \`package.json\` scripts.\n" "$script"
          found=1
        fi
      done
      if has_path "pnpm-lock.yaml"; then
        printf -- "- \`pnpm install\` lockfile observed: \`pnpm-lock.yaml\`.\n"
      elif has_path "yarn.lock"; then
        printf -- "- \`yarn install\` lockfile observed: \`yarn.lock\`.\n"
      elif has_path "package-lock.json"; then
        printf -- "- \`npm ci\` lockfile observed: \`package-lock.json\`.\n"
      fi
    fi

    if has_path "pom.xml"; then
      printf -- "- \`mvn test\` candidate from \`pom.xml\`.\n"
      found=1
    fi

    if has_path "build.gradle" || has_path "build.gradle.kts"; then
      if has_path "gradlew"; then
        printf -- "- \`./gradlew test\` candidate from Gradle wrapper.\n"
      else
        printf -- "- \`gradle test\` candidate from Gradle build file.\n"
      fi
      found=1
    fi

    if has_path "pyproject.toml" || has_path "requirements.txt" || has_path "setup.py"; then
      printf -- "- Python project indicators observed. Confirm local test command before relying on it.\n"
      if has_dir "tests"; then
        printf -- "- \`pytest\` may be relevant because \`tests/\` exists.\n"
      fi
      found=1
    fi

    [ "$found" -eq 1 ] || printf -- "- No validation commands inferred. Add manual notes below.\n"

    printf "\n## Smoke Candidates\n\n"
    if has_glob "vite.config.*"; then
      printf -- "- Vite config observed. Common smoke URL: \`APP_URL=http://localhost:5173 bash .harness/scripts/smoke.sh\`.\n"
    elif has_glob "next.config.*"; then
      printf -- "- Next config observed. Common smoke URL: \`APP_URL=http://localhost:3000 bash .harness/scripts/smoke.sh\`.\n"
    else
      printf -- "- No UI runtime config observed by this script.\n"
    fi
  } > "$out"
}

write_project_map() {
  local out="$1"

  {
    printf "Generated at: \`%s\`\n\n" "$(date -Iseconds)"
    printf "Discovery output records observed project layout only.\n\n"

    printf "## General Files and Directories\n\n"
    found=0
    for item in README.md docs src app tests e2e; do
      if has_path "$item"; then print_fact "$item" "$item"; found=1; fi
    done
    [ "$found" -eq 1 ] || printf -- "- No general project indicators observed.\n"

    printf "\n## Top-Level Directories\n\n"
    found=0
    for dir in "$ROOT_DIR"/*; do
      [ -d "$dir" ] || continue
      rel="${dir#$ROOT_DIR/}"
      case "$rel" in
        .git|.harness) continue ;;
      esac
      printf -- "- \`%s/\`\n" "$rel"
      found=1
    done
    [ "$found" -eq 1 ] || printf -- "- No top-level source directories observed.\n"
  } > "$out"
}

main() {
  local tmp_stack tmp_validation tmp_map

  mkdir -p "$PROJECT_DIR"
  tmp_stack="$(mktemp)"
  tmp_validation="$(mktemp)"
  tmp_map="$(mktemp)"

  trap "rm -f '$tmp_stack' '$tmp_validation' '$tmp_map'" EXIT

  write_stack_profile "$tmp_stack"
  write_validation_profile "$tmp_validation"
  write_project_map "$tmp_map"

  update_generated_section "$PROJECT_DIR/STACK_PROFILE.md" "Stack Profile" "$tmp_stack"
  update_generated_section "$PROJECT_DIR/VALIDATION_PROFILE.md" "Validation Profile" "$tmp_validation"
  update_generated_section "$PROJECT_DIR/PROJECT_MAP.md" "Project Map" "$tmp_map"

  echo "Updated project discovery files:"
  echo "  $PROJECT_DIR/STACK_PROFILE.md"
  echo "  $PROJECT_DIR/VALIDATION_PROFILE.md"
  echo "  $PROJECT_DIR/PROJECT_MAP.md"
}

main "$@"
