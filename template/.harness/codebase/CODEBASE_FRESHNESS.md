# Codebase Freshness

Purpose: track freshness metadata for `.harness/codebase/*`.

Do not implement cron or automation here. Future freshness checker work is TODO only.

## Last Reviewed

- Date: `<YYYY-MM-DD or unknown>`
- Reviewer: `<agent/user or unknown>`
- Last known commit/hash: `<git hash or unavailable>`
- Review scope: `<areas/files inspected>`
- Confidence: `<high/medium/low>`

## Known Stale Areas

| Area/doc | Why stale | What to inspect next |
|---|---|---|
| `<area/doc>` | `<reason>` | `<paths/searches>` |

## Freshness Triggers

Review `.harness/codebase/*` when:

- Source directories, routes, commands, jobs, or major modules are added/removed.
- Shared functions/classes/schemas/routes are renamed.
- Tests move or new test boundaries appear.
- Codebase docs contradict source evidence during a coding run.

## TODO

- Add a future periodic freshness checker or cron that can detect moved/deleted indexed paths.
- Add a future report that compares `CODEBASE_SOURCE_EVIDENCE.md` with current `git` state.
