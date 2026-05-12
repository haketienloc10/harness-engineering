# Codebase Entrypoints

Purpose: list concrete code entrypoints that start execution or route work into the source tree.

Do not record project stack/runtime/package-manager facts here. Reference `.harness/project/PROJECT_PROFILE.md` for that context.

## Entrypoints

| Entrypoint | Path | Purpose | Inspect before editing |
|---|---|---|---|
| Frontend root | `<path>` | `<what starts here>` | `<routes, providers, layout, state, tests>` |
| API route | `<path>` | `<request/action handled>` | `<validation, service calls, callers, tests>` |
| Backend server | `<path>` | `<server startup or request dispatch>` | `<middleware, routes, config/env reads, tests>` |
| CLI command | `<path>` | `<command or bin entry>` | `<argument parsing, side effects, command tests>` |
| Worker/job | `<path>` | `<job trigger or worker loop>` | `<queue/event source, retries, idempotency, tests>` |

## Notes

- Keep entries path-specific and source-level.
- Prefer exact files over broad directories when an entrypoint is known.
- If an entrypoint no longer exists, remove or mark it stale in `CODEBASE_FRESHNESS.md`.
