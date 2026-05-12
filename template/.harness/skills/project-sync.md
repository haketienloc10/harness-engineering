# Project Sync

This Harness workflow skill is project-local and agent-agnostic. It is loaded only after selection from `.harness/HARNESS_SKILLS.md`.

Use this skill to inspect the installed Harness context and the real host project, then create or update `.harness/project/*` with evidence-backed project context.

## Rules

- Preserve correct local context, manual notes, and decisions.
- Correct stale, contradicted, or low-confidence context when current evidence supports the correction.
- Record uncertainty explicitly instead of guessing.
- Do not modify application source code.
- Do not overwrite user-owned project context blindly.
- Keep edits limited to `.harness/project/*` unless the user explicitly asks for something else.

## 1. Check Current Harness State First

Read these files before inspecting the host project:

```txt
.harness/project/*
.harness/runs/RUN_INDEX.md
.harness/backlog/HARNESS_BACKLOG.md
.harness/HARNESS_SKILLS.md
AGENTS.md
AGENTS.harness.md
```

If a file is missing, record that as evidence. Do not treat missing files as an error unless they block the requested work.

From the current Harness state, identify:

- Existing project facts and manual notes.
- Active or recent root runs, Epic containers, and child runs that may reveal project context.
- Backlog items that mention local workflow or project constraints.
- Local instructions that affect context, verification, or source boundaries.
- Contradictions, stale assumptions, and low-confidence claims.

## 2. Inspect The Real Host Project

Inspect files and directories outside `.harness/`. Prefer source evidence over assumptions.

Recommended evidence sources:

- Repository root files such as `README.md`, package manifests, lockfiles, build files, workspace config, runtime config, CI config, Docker files, and environment examples.
- Source directories, test directories, app routes, command entry points, service entry points, scripts, schemas, migrations, API specs, and docs.
- Existing test and build scripts.
- Current dependency and package-manager evidence.
- Naming, module structure, domain models, and user-facing flows visible in code and docs.

Avoid:

- Treating generated files or dependency directories as source.
- Reading large vendored, build, cache, or lockfile content unless needed for package-manager evidence.
- Modifying application source code.

## 3. Detect Required Context

Detect and document:

- Project type.
- Stack.
- Package manager.
- Runtime.
- Entry points.
- Source boundaries.
- Non-source boundaries.
- Verification commands.
- Architecture/module map.
- Business/domain context.
- Open questions.

Use explicit evidence for each important conclusion. When evidence is partial, mark the confidence and explain what would confirm it.

## 4. Compare Existing Context With Evidence

Compare existing `.harness/project/*` content with current evidence.

Classify each important fact as:

- `confirmed`: existing context matches current evidence.
- `updated`: existing context was stale or incomplete and has been corrected.
- `contradicted`: existing context conflicts with current evidence.
- `uncertain`: evidence is insufficient or ambiguous.
- `new`: fact was not previously recorded.

Preserve correct local context even if it is not directly rediscovered during this pass, especially manual notes, local decisions, known flaky checks, and user-provided constraints.

## 5. Update Project Context Files

Create or update these files:

```txt
.harness/project/PROJECT_PROFILE.md
.harness/project/PROJECT_CONTEXT.md
.harness/project/PROJECT_RULES.md
.harness/project/PROJECT_VERIFICATION.md
.harness/project/PROJECT_ARCHITECTURE.md
.harness/project/PROJECT_GLOSSARY.md
.harness/project/PROJECT_OPEN_QUESTIONS.md
```

Use concise, evidence-backed content. Include "Last reviewed" or equivalent review metadata where helpful.

Recommended file responsibilities:

- `PROJECT_PROFILE.md`: project type, stack, package manager, runtime, entry points, repository layout summary.
- `PROJECT_CONTEXT.md`: product purpose, business/domain context, user flows, external systems, important docs.
- `PROJECT_RULES.md`: local engineering rules, source boundaries, non-source boundaries, ownership constraints, agent instructions.
- `PROJECT_VERIFICATION.md`: install commands, lint/typecheck/test/build/smoke commands, known gaps, flaky checks, required evidence.
- `PROJECT_ARCHITECTURE.md`: module map, boundaries, dependencies, data flow, integration points, risk areas.
- `PROJECT_GLOSSARY.md`: domain terms, acronyms, code names, project-specific language.
- `PROJECT_OPEN_QUESTIONS.md`: unresolved questions, missing evidence, assumptions needing user confirmation.

If old project adapter files exist, such as `PROJECT_MAP.md`, `SOURCE_OF_TRUTH.md`, `STACK_PROFILE.md`, `VALIDATION_PROFILE.md`, `MODULE_MAP.md`, or `LOCAL_DECISIONS.md`, treat them as existing local context. Migrate useful facts into the current files, but do not delete old files unless the user explicitly asks.

## 6. Preserve And Correct Safely

When updating files:

- Preserve correct manual notes and local decisions.
- Keep useful historical context if it still affects current work.
- Correct stale or contradicted context with a short evidence note.
- Mark low-confidence claims instead of presenting them as fact.
- Add open questions when evidence is insufficient.
- Avoid broad formatting churn unrelated to contextualization.

## 7. Final Report

Return a concise final report with:

- Files created or updated.
- Key facts confirmed or changed.
- Verification commands identified.
- Open questions that still need user input.
- Any context that was preserved because it appears local or user-authored.

Do not claim the project is fully understood if important evidence is missing.
