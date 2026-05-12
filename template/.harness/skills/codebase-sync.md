# Codebase Sync

This Harness workflow skill is project-local and agent-agnostic. It is loaded only after selection from `.harness/HARNESS_SKILLS.md`.

Use this skill to create or update `.harness/codebase/*` as a lightweight source-navigation and change-impact map for future coding runs.

## Hard Boundary

- Do not duplicate `.harness/project/*`.
- If a fact belongs in `.harness/project/*`, reference the relevant project file instead of copying it.
- Only write source-navigation, concrete entrypoints, technical flows, source areas, change impact, source evidence, and freshness metadata.
- Do not modify application source code.
- Do not overwrite user-authored codebase notes blindly; preserve correct local notes and correct stale notes only when source evidence supports the change.

Project-owned facts stay in:

- `.harness/project/PROJECT_PROFILE.md`: stack, runtime, package manager, broad repository profile.
- `.harness/project/PROJECT_CONTEXT.md`: product/business/domain context and user/product flows.
- `.harness/project/PROJECT_RULES.md`: local engineering rules and source/non-source policy.
- `.harness/project/PROJECT_VERIFICATION.md`: general verification commands and evidence expectations.
- `.harness/project/PROJECT_ARCHITECTURE.md`: high-level architecture and module boundaries.
- `.harness/project/PROJECT_GLOSSARY.md`: domain language.
- `.harness/project/PROJECT_OPEN_QUESTIONS.md`: unresolved project-level questions.

## 1. Read Existing Context

Read, if present:

```txt
.harness/project/*
.harness/codebase/*
.harness/HARNESS_SKILLS.md
AGENTS.md
AGENTS.harness.md
```

Use `.harness/project/*` to avoid duplicating project-level context. If project context is missing, stale, contradictory, or low-confidence, run or recommend `project-sync` first unless the user explicitly asked for a narrow codebase-only refresh.

## 2. Inspect The Real Source Tree

Inspect files and directories outside `.harness/`. Prefer concrete source evidence over assumptions.

Recommended source evidence:

- Source directories and module boundaries as they exist on disk.
- Frontend roots, route files, API route handlers, server entries, CLI bins, workers, jobs, and scripts.
- Imports/usages around shared functions, classes, schemas, hooks, services, routes, commands, and generated contracts.
- Tests colocated with or exercising the source areas.
- Build/runtime config only when needed to locate entrypoints; record stack/runtime facts in `.harness/project/*`, not `.harness/codebase/*`.

Avoid:

- Generated, vendored, dependency, build, and cache directories unless they are needed to understand a source boundary.
- Copying business/product descriptions, stack summaries, general verification command lists, integration inventories, or open questions from `.harness/project/*`.

## 3. Update Codebase Files

Create or update only this file set:

```txt
.harness/codebase/CODEBASE_INDEX.md
.harness/codebase/CODEBASE_AREAS.md
.harness/codebase/CODEBASE_ENTRYPOINTS.md
.harness/codebase/CODEBASE_FLOWS.md
.harness/codebase/CODEBASE_CHANGE_IMPACT.md
.harness/codebase/CODEBASE_SOURCE_EVIDENCE.md
.harness/codebase/CODEBASE_FRESHNESS.md
```

Do not create:

```txt
.harness/codebase/CODEBASE_PROFILE.md
.harness/codebase/CODEBASE_DATA_MODEL.md
.harness/codebase/CODEBASE_INTEGRATIONS.md
.harness/codebase/CODEBASE_VERIFICATION.md
.harness/codebase/CODEBASE_OPEN_QUESTIONS.md
```

## 4. File Responsibilities

- `CODEBASE_INDEX.md`: first file to read before non-trivial coding tasks; map feature/task keywords to source areas, relevant codebase docs, and likely source files/folders to inspect. Reference project profile and verification files instead of duplicating them.
- `CODEBASE_AREAS.md`: source areas/modules by actual source tree, with responsibility, important files/folders, common edit targets, related tests, and coupling/risk notes.
- `CODEBASE_ENTRYPOINTS.md`: concrete frontend/API/backend/CLI/worker/job entrypoints, with path, purpose, and what to inspect before editing.
- `CODEBASE_FLOWS.md`: technical source-level flows showing how a user/API/system action moves through files/modules.
- `CODEBASE_CHANGE_IMPACT.md`: impact map for coding, including affected files, tests, usages to search, and regression risks.
- `CODEBASE_SOURCE_EVIDENCE.md`: inspected source paths, important files reviewed, confidence, and stale/uncertain source areas.
- `CODEBASE_FRESHNESS.md`: last reviewed metadata, last known commit/hash if available, known stale areas, and TODO for a future periodic freshness checker.

## 5. Sync Method

1. Build a source-area shortlist from current source paths.
2. Identify concrete entrypoints.
3. Trace only important technical flows that help future coding runs.
4. For each source area, record change-impact checks and usage searches.
5. Record evidence and confidence.
6. Update freshness metadata, including the current `git rev-parse --short HEAD` result when available.

## 6. Preservation Rules

- Preserve correct local notes and manual annotations.
- Correct paths or impact notes that source evidence proves stale.
- Mark uncertain areas explicitly when evidence is incomplete.
- Keep content concise and navigational.
- Prefer tables and short checklists over long prose.

## 7. Final Report

Return a concise final report with:

- `.harness/codebase/*` files created or updated.
- Important source areas, entrypoints, and impact risks added or changed.
- Stale or uncertain areas that remain.
- Any project-level facts intentionally left in `.harness/project/*`.
