# Harness Template Verification Evidence

Date: 2026-05-11

## Summary

Result: Pass

This evidence covers the requested verification after separating the seed repository from the installed Harness template.

## Code Change Verified

Minor fix in `template/.harness/scripts/inspect-project.sh`:

- `update_generated_section` no longer creates a `mktemp` file when the target project adapter file does not exist.
- `mktemp` is now created only in branches that rewrite an existing file.

## Commands Executed

```bash
bash -n scripts/install-harness.sh
bash -n template/.harness/scripts/*.sh
```

Result:

```text
PASS bash -n scripts/install-harness.sh
PASS bash -n template/.harness/scripts/*.sh
```

## Temp Install

Command shape:

```bash
tmp="$(mktemp -d)"
cd "$tmp"
git init
echo "# demo" > README.md
bash /home/locdt/harness-engineering/template/.harness/scripts/install.sh \
  --target "$tmp" \
  --agents-mode merge \
  --yes
```

Checked:

```bash
test -f AGENTS.md
test -d .harness/guides
test -d .harness/templates
test -d .harness/project-templates
test -d .harness/project
test -f .harness/project/PROJECT_MAP.md
test -f .harness/project/SOURCE_OF_TRUTH.md
test -f .harness/project/STACK_PROFILE.md
test -f .harness/project/VALIDATION_PROFILE.md
test -f .harness/project/MODULE_MAP.md
test -f .harness/project/LOCAL_DECISIONS.md
test -f .harness/runs/RUN_INDEX.md
test -f .harness/backlog/HARNESS_BACKLOG.md
test -f .harness/INSTALLATION.md
```

Result:

```text
PASS temp install: /tmp/tmp.4L1qZrWu8K
```

## Idempotency / Ownership-Safe Update

Command shape:

```bash
echo "MANUAL LOCAL DECISION" >> .harness/project/LOCAL_DECISIONS.md
echo "LOCAL BACKLOG" >> .harness/backlog/HARNESS_BACKLOG.md
echo "| RUN-LOCAL | test | completed | main | | user | today | today |" >> .harness/runs/RUN_INDEX.md

bash /home/locdt/harness-engineering/template/.harness/scripts/install.sh \
  --target "$tmp" \
  --agents-mode merge \
  --yes \
  --force

grep "MANUAL LOCAL DECISION" .harness/project/LOCAL_DECISIONS.md
grep "LOCAL BACKLOG" .harness/backlog/HARNESS_BACKLOG.md
grep "RUN-LOCAL" .harness/runs/RUN_INDEX.md
```

Result:

```text
PASS idempotency ownership-safe update
```

## Inspect Project Manual Note Preservation

Command shape:

```bash
echo "MANUAL STACK NOTE" >> .harness/project/STACK_PROFILE.md
bash .harness/scripts/inspect-project.sh
grep "MANUAL STACK NOTE" .harness/project/STACK_PROFILE.md
grep "HARNESS:GENERATED:START" .harness/project/STACK_PROFILE.md
```

Result:

```text
PASS inspect-project preserves manual notes and generated marker
```

## Preserve Mode

Command shape:

```bash
tmp2="$(mktemp -d)"
cd "$tmp2"
git init
echo "Existing agent instruction" > AGENTS.md

bash /home/locdt/harness-engineering/template/.harness/scripts/install.sh \
  --target "$tmp2" \
  --agents-mode preserve \
  --yes

grep "Existing agent instruction" AGENTS.md
test -f AGENTS.harness.md
```

Result:

```text
PASS preserve mode: /tmp/tmp.S17Oexbqt3
```

## Dry Run

Command shape:

```bash
tmp3="$(mktemp -d)"
bash /home/locdt/harness-engineering/template/.harness/scripts/install.sh \
  --target "$tmp3" \
  --agents-mode merge \
  --dry-run

test ! -d "$tmp3/.harness"
```

Result:

```text
PASS dry-run no .harness created: /tmp/tmp.npIjr1HmbA
```

## Default `--yes` Merge Safety

Additional safety check:

```bash
tmp4="$(mktemp -d)"
echo "Existing agent instruction" > "$tmp4/AGENTS.md"

bash /home/locdt/harness-engineering/template/.harness/scripts/install.sh \
  --target "$tmp4" \
  --yes

grep "Existing agent instruction" "$tmp4/AGENTS.md"
grep "Existing Repository Instructions" "$tmp4/AGENTS.md"
```

Result:

```text
PASS --yes without agents-mode merges existing AGENTS.md: /tmp/tmp.Uv7Q4UauL7
```

## Tmp File Regression

Regression check for the minor issue in `inspect-project.sh`:

```bash
before_count="$(find /tmp -maxdepth 1 -type f -name 'tmp.*' | wc -l | tr -d ' ')"
tmp5="$(mktemp -d)"
bash /home/locdt/harness-engineering/template/.harness/scripts/install.sh \
  --target "$tmp5" \
  --agents-mode merge \
  --yes
rm -f "$tmp5/.harness/project/STACK_PROFILE.md" \
  "$tmp5/.harness/project/VALIDATION_PROFILE.md" \
  "$tmp5/.harness/project/PROJECT_MAP.md"
bash "$tmp5/.harness/scripts/inspect-project.sh"
after_count="$(find /tmp -maxdepth 1 -type f -name 'tmp.*' | wc -l | tr -d ' ')"
test "$before_count" = "$after_count"
```

Result:

```text
tmp files before=6 after=6
PASS no extra /tmp tmp.* files when project files are missing: /tmp/tmp.ARM8HRY1DN
```

## Final Result

```text
ALL PASS
```
