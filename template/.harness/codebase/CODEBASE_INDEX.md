# Codebase Index

Purpose: first codebase document to read before non-trivial coding tasks.

This file maps task language to source areas, focused codebase docs, and likely source files or folders to inspect.

Do not record stack, runtime, package manager, product context, business rules, or general verification here. Use:

- `.harness/project/PROJECT_PROFILE.md` for stack, runtime, package manager, and broad repository profile.
- `.harness/project/PROJECT_VERIFICATION.md` for general verification commands and evidence expectations.

## How To Use

1. Find the closest keyword or task shape below.
2. Read only the linked `.harness/codebase/*` files that help locate source.
3. Inspect the actual source files before editing.
4. Search usages before changing existing functions, classes, routes, commands, or exported APIs.
5. If source evidence contradicts this document, update this file or mark stale context in `CODEBASE_FRESHNESS.md`.

## Keyword Map

| Task keywords | Source area | Read next | Inspect first |
|---|---|---|---|
| `<keyword>` | `<source area>` | `CODEBASE_AREAS.md`, `CODEBASE_CHANGE_IMPACT.md` | `<files/folders>` |

## Common Task Shapes

| Task shape | Read next | Source inspection checklist |
|---|---|---|
| Add or change a UI route/view | `CODEBASE_ENTRYPOINTS.md`, `CODEBASE_FLOWS.md`, `CODEBASE_CHANGE_IMPACT.md` | Locate route root, component tree, state/data hooks, related tests |
| Add or change an API route/action | `CODEBASE_ENTRYPOINTS.md`, `CODEBASE_FLOWS.md`, `CODEBASE_CHANGE_IMPACT.md` | Locate route handler, validation, service/module calls, callers, tests |
| Change a shared module/helper | `CODEBASE_AREAS.md`, `CODEBASE_CHANGE_IMPACT.md` | Search imports/usages, inspect consumers, check regression risks |
| Change a CLI/job/worker path | `CODEBASE_ENTRYPOINTS.md`, `CODEBASE_FLOWS.md`, `CODEBASE_CHANGE_IMPACT.md` | Locate command/job entry, config/env reads, side effects, tests |

## Source Area Shortlist

| Source area | Purpose | Likely files/folders | Notes |
|---|---|---|---|
| `<area>` | `<source-level responsibility>` | `<paths>` | `<navigation or risk note>` |

## Staleness Signals

- File paths listed here no longer exist.
- New routes, commands, jobs, or source folders are not indexed.
- Impact notes miss current tests or major consumers.
- `CODEBASE_SOURCE_EVIDENCE.md` shows low confidence for the relevant area.
