# Harness Workflow Skills

Agents should read this registry first.

Select a relevant Harness workflow skill by name, description, and trigger conditions.

Load only the selected skill file.

Do not load all skill files by default.

## project-sync

Description:
Inspect current Harness context and the host project, then create or update `.harness/project/*` with the latest evidence.

Use when:
- Harness was just installed.
- User asks to contextualize, refresh, correct, or update project context.
- `.harness/project/*` is missing, stale, contradictory, or low-confidence.
- Before a major run/epic if project context is not reliable.
- After a run/epic if new project facts were discovered.

Load:
`.harness/skills/project-sync.md`

Outputs:
- Updated `.harness/project/*`
- Summary of facts updated
- Open questions if evidence is insufficient
