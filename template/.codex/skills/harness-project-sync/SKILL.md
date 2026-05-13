# Harness Project Sync

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
AGENTS.md
AGENTS.harness.md
```

If a file is missing, record that as evidence. Do not treat missing files as an error unless they block the requested work.

Identify existing project facts, active/recent runs, backlog constraints, local instructions, contradictions, stale assumptions, and low-confidence claims.

## 2. Inspect The Real Host Project

Inspect files and directories outside `.harness/`. Prefer source evidence over assumptions.

Recommended evidence sources:

- Repository root files such as `README.md`, package manifests, lockfiles, build files, workspace config, runtime config, CI config, Docker files, and environment examples.
- Source directories, test directories, app routes, command entry points, service entry points, scripts, schemas, migrations, API specs, and docs.
- Existing test and build scripts.
- Current dependency and package-manager evidence.
- Naming, module structure, domain models, and user-facing flows visible in code and docs.

Avoid generated files, dependency directories, and broad application edits.

## 3. Update Project Context Files

Create or update:

```txt
.harness/project/PROJECT_PROFILE.md
.harness/project/PROJECT_CONTEXT.md
.harness/project/PROJECT_RULES.md
.harness/project/PROJECT_VERIFICATION.md
.harness/project/PROJECT_ARCHITECTURE.md
.harness/project/PROJECT_GLOSSARY.md
.harness/project/PROJECT_OPEN_QUESTIONS.md
```

If old project adapter files exist, treat them as local context. Migrate useful facts into the current files, but do not delete old files unless explicitly asked.

## 4. Final Report

Return a concise final report with files created or updated, key facts confirmed or changed, verification commands identified, open questions, and preserved local context.
