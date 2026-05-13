# Harness Codebase Sync

Use this skill to create or update `.harness/codebase/*` as a lightweight source-navigation and change-impact map for future coding runs.

## Hard Boundary

- Do not duplicate `.harness/project/*`.
- If a fact belongs in `.harness/project/*`, reference the relevant project file instead of copying it.
- Only write source-navigation, concrete entrypoints, technical flows, source areas, change impact, source evidence, and freshness metadata.
- Do not modify application source code.
- Preserve correct local notes and correct stale notes only when source evidence supports the change.

## 1. Read Existing Context

Read, if present:

```txt
.harness/project/*
.harness/codebase/*
AGENTS.md
AGENTS.harness.md
```

Use `.harness/project/*` to avoid duplicating project-level context. If project context is missing, stale, contradictory, or low-confidence, run or recommend `harness-project-sync` first unless the user asked for a narrow codebase-only refresh.

## 2. Inspect The Real Source Tree

Inspect files and directories outside `.harness/`. Prefer concrete source evidence over assumptions.

Recommended source evidence:

- Source directories and module boundaries as they exist on disk.
- Frontend roots, route files, API route handlers, server entries, CLI bins, workers, jobs, and scripts.
- Imports/usages around shared functions, classes, schemas, hooks, services, routes, commands, and generated contracts.
- Tests colocated with or exercising the source areas.
- Build/runtime config only when needed to locate entrypoints.

Avoid generated, vendored, dependency, build, and cache directories unless they are needed to understand a source boundary.

## 3. Update Codebase Files

Create or update only:

```txt
.harness/codebase/CODEBASE_INDEX.md
.harness/codebase/CODEBASE_AREAS.md
.harness/codebase/CODEBASE_ENTRYPOINTS.md
.harness/codebase/CODEBASE_FLOWS.md
.harness/codebase/CODEBASE_CHANGE_IMPACT.md
.harness/codebase/CODEBASE_SOURCE_EVIDENCE.md
.harness/codebase/CODEBASE_FRESHNESS.md
```

Do not create project-level duplicates such as `CODEBASE_PROFILE.md`, `CODEBASE_VERIFICATION.md`, or `CODEBASE_OPEN_QUESTIONS.md`.

## 4. Sync Method

1. Build a source-area shortlist from current source paths.
2. Identify concrete entrypoints.
3. Trace only important technical flows that help future coding runs.
4. Record change-impact checks and usage searches.
5. Record evidence and confidence.
6. Update freshness metadata, including current `git rev-parse --short HEAD` when available.

## 5. Final Report

Return a concise final report with `.harness/codebase/*` files created or updated, important source areas/entrypoints/impact risks changed, stale or uncertain areas that remain, and project-level facts intentionally left in `.harness/project/*`.
