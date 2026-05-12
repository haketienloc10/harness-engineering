# Codebase Areas

Purpose: map actual source-tree areas to coding responsibilities and edit risks.

Do not restate high-level architecture already owned by `.harness/project/PROJECT_ARCHITECTURE.md`. Keep this file concrete: paths, edit targets, tests, coupling, and source-level risks.

## Areas

### `<area name>`

- Responsibility: `<what this source area does at code level>`
- Important files/folders:
  - `<path>`
- Common edit targets:
  - `<path or symbol>`
- Related tests:
  - `<test path or test command reference>`
- Coupling/risk notes:
  - `<imports, shared contracts, generated files, runtime side effects, migration risks>`

## Cross-Area Couplings

| From area | To area | Coupling | Check before editing |
|---|---|---|---|
| `<area>` | `<area>` | `<shared type/API/import/runtime path>` | `<files, searches, tests>` |

## Source Boundaries

Record concrete source boundaries only. For project-wide source/non-source policy, reference `.harness/project/PROJECT_RULES.md`.

| Boundary | Paths | Notes |
|---|---|---|
| Source to inspect | `<paths>` | `<notes>` |
| Usually avoid | `<generated/vendor/cache paths>` | `<why>` |
