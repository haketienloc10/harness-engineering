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

## runtime-role-separation

Description:
Use the Runtime Role Separation guide before non-trivial implementation to enforce separate Planner, Contract Reviewer, Generator, and Evaluator sessions and prevent production-grade self-approval.

Use when:
- Starting any non-trivial implementation run.
- Reviewing or approving an implementation contract.
- Handing work from planning to implementation or from implementation to evaluation.
- A task might otherwise be handled by one agent/session playing multiple roles.
- Recording fallback single-session mode and degraded independence.

Load:
`.harness/guides/RUNTIME_ROLE_SEPARATION.md`

Outputs:
- Correct runtime role metadata in run artifacts
- Clear handoff note between roles
- Explicit `independence: independent` or `independence: degraded`
