# Agent-First Harness

This file is the agent entrypoint. Obey it before changing the target repo.

The app is what users touch. The harness is what agents operate.

## Non-Negotiables

- Do not edit first. Classify the request first.
- Do not invent product truth. Derive it from user intent, product docs,
  stories, code, tests, and durable records.
- Do not grow a monolithic spec. Convert specs into product docs, stories,
  decisions, and proof.
- Do not skip validation silently. Run the right checks or state exactly why no
  check could run.
- Do not use optional tools by assumption. Query capability first.
- Do not ask the human for routine execution choices. Ask only when ambiguity or
  risk requires a real decision.
- Do not leave the next agent blind. Update records, docs, traces, or backlog
  friction when the task changes them.

## Required Read Order

At task start, read:

1. `AGENTS.md`
2. `_harness/HARNESS.md`
3. `_harness/FEATURE_INTAKE.md`
4. `_harness/CONTEXT_RULES.md`
5. `_harness/bin/harness-cli query matrix` when the CLI exists

Then read conditionally:

- `_harness/ARCHITECTURE.md` for code structure, boundaries, data,
  providers, runtime, public contracts, or app surfaces.
- `_harness/TOOL_REGISTRY.md` before optional external tools.
- `docs/product/*` when product behavior changes.
- `docs/stories/*` when work maps to an existing story.
- `docs/decisions/*` when architecture, source hierarchy, durable
  records, validation, or high-risk behavior changes.
- `_harness/templates/*` before creating harness artifacts.

If a required harness file is missing, continue only if the task is safe from
local context. Record the missing file as harness friction.

## CLI Contract

Use the repository-local CLI as the durable layer:

```bash
_harness/bin/harness-cli <command>
```

Windows:

```powershell
.\_harness\bin\harness-cli.ps1 <command>
```

If `harness.db` is missing and the CLI exists, run:

```bash
_harness/bin/harness-cli init
```

Record operational state through the CLI whenever possible:

```bash
_harness/bin/harness-cli intake --type <type> --summary <text> --lane <lane>
_harness/bin/harness-cli query matrix
_harness/bin/harness-cli tool check
_harness/bin/harness-cli story add --id <id> --title <text> --lane <lane>
_harness/bin/harness-cli story update --id <id> --unit 1 --integration 1 --e2e 0 --platform 0
_harness/bin/harness-cli story verify <id>
_harness/bin/harness-cli story verify-all
_harness/bin/harness-cli decision add --id <id> --title <text> --doc docs/decisions/<file>.md
_harness/bin/harness-cli backlog add --title <text> --pain <text> --risk tiny
_harness/bin/harness-cli trace --summary <text> --agent codex --outcome completed
```

If the CLI is unavailable, use markdown artifacts and report the missing CLI as
friction.

## Work Loop

For every task, execute this order:

1. Classify input type.
2. Run the risk checklist.
3. Select lane: `tiny`, `normal`, or `high-risk`.
4. Record intake when the CLI exists.
5. Locate affected product docs, stories, decisions, code, and tests.
6. Query proof matrix when the CLI exists.
7. Run `tool check`, then query optional tool capability before external tool
   use.
8. Implement the smallest safe slice for the lane.
9. Update product docs, story state, proof, decisions, and templates if changed.
10. Validate according to lane.
11. Record trace when the CLI exists.
12. Fix harness friction immediately or record backlog.

## Input Types

Use exactly one:

| Type                  | Use when                                                                     |
| --------------------- | ---------------------------------------------------------------------------- |
| `New spec`            | User supplies a project spec or large product idea.                          |
| `Spec slice`          | A selected behavior from an accepted spec is ready.                          |
| `Change request`      | Accepted behavior changes, breaks, or needs refinement.                      |
| `New initiative`      | Multiple stories are needed.                                                 |
| `Maintenance request` | Dependencies, architecture, performance, security, CI, or operations change. |
| `Harness improvement` | Agent workflow, templates, proof, tool registry, or instructions change.     |

## Risk Lanes

### Tiny

Use only for low-risk docs, copy, naming, narrow edits, or limited setup without
domain schema, CRUD behavior, auth, authorization, provider integration, or
migration behavior.

Do:

- Record intake if CLI exists.
- Patch directly.
- Run quick checks.
- Update changed docs.
- Record friction only if found.

### Normal

Use for story-sized behavior with bounded blast radius.

Do:

- Create or update one story from `_harness/templates/story.md` when the
  work is behavior-bearing.
- Link relevant product docs.
- Add or update validation expectations.
- Implement the smallest vertical slice.
- Update durable story status and proof when CLI exists.
- Record a Standard trace.

### High-Risk

Use when the work can affect security, data, scope, public contracts, multiple
roles/platforms, or validation guarantees.

Do:

- Create a high-risk packet from `_harness/templates/high-risk-story/`.
- Fill `execplan.md`, `overview.md`, `design.md`, and `validation.md`.
- Read relevant decisions before implementation.
- Ask the human only when product or safety direction is ambiguous.
- Add a durable decision for meaningful behavior, architecture, authorization,
  data ownership, API shape, or validation changes.
- Record a Detailed trace.

## Risk Checklist

Mark every applicable flag:

| Risk flag         | Applies when touched                                            |
| ----------------- | --------------------------------------------------------------- |
| Auth              | login, logout, sessions, JWT, passwords, refresh tokens         |
| Authorization     | roles, permissions, tenant/company/workspace scope              |
| Data model        | schema, migrations, uniqueness, deletion, retention             |
| Audit/security    | audit logs, privacy, sensitive data, access logs                |
| External systems  | email, payments, cloud services, SDKs, queues, webhooks         |
| Public contracts  | API shape, response envelope, client-visible behavior           |
| Cross-platform    | desktop/mobile/browser split, native shell behavior, deep links |
| Existing behavior | implemented or test-covered behavior changes                    |
| Weak proof        | unclear or missing tests around the affected area               |
| Multi-domain      | more than one product domain changes                            |

Classification:

- `0-1` flags: `tiny` or `normal`, based on code impact.
- `2-3` flags: `normal` with stronger validation.
- `4+` flags: `high-risk`.
- Any hard gate is `high-risk` unless the human explicitly narrows scope.

Hard gates:

- Auth.
- Authorization.
- Data loss or migration.
- Audit/security.
- External provider behavior.
- Removing or weakening validation.

## Source Hierarchy

When sources conflict, use this order:

```text
Current user instruction
  -> docs/product/*
  -> docs/stories/*
  -> _harness/bin/harness-cli query matrix
  -> docs/decisions/* plus CLI decisions
  -> code and tests
  -> historical specs or examples
```

User-provided specs are input material. Living truth belongs in product docs,
stories, proof, and decisions.

## Architecture Rules

- Create stack folders only when a selected story needs them.
- Keep inner layers independent from outer layers.
- Parse unknown data at boundaries before inner code receives it.
- Keep commands and queries separate when the product has reads and writes.
- Treat audit logs as product records and application logs as operational
  records.

Default dependency direction:

```text
domain
  <- application
      <- infrastructure
          <- interface
              <- app surfaces
```

## Tool Rule

Before any optional external tool, run:

```bash
_harness/bin/harness-cli tool check
_harness/bin/harness-cli query tools --capability <capability> --status present
```

If no provider is registered, cleanly skip and note
`capability <name>: inactive` in the trace. If a provider is registered but
missing, mark `Weak proof` and record the gap.

## Validation Rule

- Tiny: run quick checks available for the touched files.
- Normal: run focused tests and configured story verification.
- High-risk: run the validation plan from the high-risk packet and explain any
  skipped proof.

Record proof status with:

```bash
_harness/bin/harness-cli story verify <story-id>
_harness/bin/harness-cli story update --id <story-id> --unit 1 --integration 1 --e2e 0 --platform 0 --evidence "<commands run>"
_harness/bin/harness-cli query matrix
```

Do not claim completion without proof or an explicit validation gap.

## Trace Rule

Before final response, record a trace when CLI exists.

Minimum by lane:

| Lane        | Trace tier                                                   |
| ----------- | ------------------------------------------------------------ |
| `tiny`      | Minimal; Standard if harness docs or durable records changed |
| `normal`    | Standard                                                     |
| `high-risk` | Detailed                                                     |

Trace decisions do not replace decision records.

## Human Interaction Rule

Proceed without asking when a conservative, repo-consistent choice exists.

Ask only when:

- Product behavior is ambiguous and outcomes materially differ.
- High-risk implementation needs a security, data, authorization, provider, or
  public-contract decision.
- Credentials, private systems, paid actions, destructive operations, or scope
  expansion are required.
- The user explicitly requested approval before edits.

Ask one concrete question. Then continue.

## Final Response Rule

Before final response:

- Run `git status --short`.
- Confirm validation evidence or name the gap.
- Confirm changed harness artifacts when relevant.
- Confirm trace/friction status when CLI exists.

Respond concisely: changed surface, validation, durable records, and remaining
gap only.
