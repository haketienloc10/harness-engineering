# Codebase Flows

Purpose: document technical source-level flows through files and modules.

Do not duplicate product, user-journey, or business flow descriptions from `.harness/project/PROJECT_CONTEXT.md`. This file is about how a concrete action moves through source files.

## Flow Template

### `<technical flow name>`

- Trigger: `<user/API/system action at source level>`
- Entrypoint: `<path>`
- Path through source:
  1. `<path or symbol>`: `<technical role>`
  2. `<path or symbol>`: `<technical role>`
  3. `<path or symbol>`: `<technical role>`
- Data/control transfers:
  - `<event, call, prop, hook, message, query, or returned value>`
- Error/loading/edge handling:
  - `<paths or symbols to inspect>`
- Tests/evidence:
  - `<test paths, smoke path, source evidence>`
- Impact notes:
  - `<what else to inspect when changing this flow>`

## Known Flows

### `<flow name>`

- Trigger: `<trigger>`
- Entrypoint: `<path>`
- Path through source:
  1. `<path>`: `<role>`
- Data/control transfers:
  - `<transfer>`
- Error/loading/edge handling:
  - `<paths>`
- Tests/evidence:
  - `<tests or evidence>`
- Impact notes:
  - `<risks and related areas>`
