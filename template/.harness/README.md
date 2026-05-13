# Installed Harness

Thư mục này là Harness đã được cài vào target repository. Target repository sở hữu nội dung này sau install.

## Index

```txt
AGENTS.md                         bootstrap instruction
.codex/agents/*.toml              canonical lifecycle role definitions
.codex/skills/harness-*/SKILL.md  workflow skills
.harness/guides/                  policy details
.harness/workflows/               lifecycle state transitions
.harness/templates/               run and Epic artifact templates
.harness/project/                 target-owned project context
.harness/codebase/                target-owned source-navigation context
.harness/scripts/                 helper scripts
.harness/backlog/                 local Harness backlog
.harness/runs/                    normal runs, Epics, and child runs
```

## Canonical Sources

- Bootstrap: `AGENTS.md`
- Agents: `.codex/agents/*.toml`
- Skills: `.codex/skills/harness-*/SKILL.md`
- Lifecycle policy: `.harness/guides/SUBAGENT_EXECUTION.md`
- State transitions: `.harness/workflows/default-lifecycle.md`
- Verification: `.harness/scripts/verify.sh`

## Sau Khi Install

Ask your agent:

```txt
Use .codex/skills/harness-project-sync/SKILL.md to refresh project context.
Then use .codex/skills/harness-codebase-sync/SKILL.md if source-navigation or change-impact docs are missing or stale.
```
