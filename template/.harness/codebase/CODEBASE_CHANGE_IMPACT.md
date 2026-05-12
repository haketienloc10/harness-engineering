# Codebase Change Impact

Purpose: impact map for coding. When changing source area X, inspect/check Y.

This is the main reason the codebase layer exists. Keep it concrete and source-backed.

For general verification commands, reference `.harness/project/PROJECT_VERIFICATION.md` instead of duplicating them here.

## Impact Map

| Changing source area | Inspect/check before edit | Search usages | Likely affected files | Related tests | Regression risks |
|---|---|---|---|---|---|
| `<area/path/symbol>` | `<files/folders/contracts>` | `<rg pattern or symbol names>` | `<paths>` | `<test paths or focused checks>` | `<risk>` |

## Shared Contract Checklist

Use when changing exported functions, classes, schemas, routes, command flags, event names, generated types, or shared UI/state modules.

- Search direct imports/usages.
- Search string-based usages when names are used dynamically.
- Inspect tests around all major consumers.
- Check call sites for assumptions about shape, errors, loading state, side effects, or ordering.
- Update `CODEBASE_SOURCE_EVIDENCE.md` when new important source evidence is discovered.

## High-Risk Change Patterns

| Pattern | Extra inspection | Focused regression checks |
|---|---|---|
| Shared API/type/schema change | `<consumers, validators, serializers>` | `<focused tests>` |
| Routing/navigation change | `<entrypoints, links, redirects, guards>` | `<route/view tests or smoke>` |
| State/cache/data-fetching change | `<callers, invalidation, loading/error states>` | `<state and integration tests>` |
| CLI/job side effect change | `<config/env reads, retries, idempotency>` | `<command/job tests>` |
