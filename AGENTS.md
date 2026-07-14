# Agent-First Harness

Entrypoint for agents. The app is what users touch; the harness is what agents
operate. Do not edit before intake.

## Start Every Task

Read in order:

1. `AGENTS.md`
2. `_harness/HARNESS.md`
3. `_harness/FEATURE_INTAKE.md`
4. `_harness/CONTEXT_RULES.md`
5. `_harness/bin/harness-cli query matrix` when the CLI exists

Then read only what the lane and task require:

- `_harness/ARCHITECTURE.md` for structure, boundaries, data, providers,
  runtime, public contracts, or app surfaces.
- `_harness/TOOL_REGISTRY.md` before optional external tools.
- `docs/product/*` when product behavior changes.
- `docs/stories/*` when work maps to a story.
- `docs/decisions/*` when architecture, source hierarchy, durable records,
  validation, or high-risk behavior changes.
- `_harness/templates/*` before creating harness artifacts.

If `harness.db` is missing and the CLI exists, run:

```bash
_harness/bin/harness-cli init
```

If the CLI is unavailable, use markdown artifacts and record the missing CLI as
harness friction.

## Non-Negotiables

- Classify first: input type, risk flags, lane.
- Derive product truth from current user intent, product docs, stories,
  decisions, matrix proof, code, and tests.
- Convert specs into product docs, stories, decisions, and proof; do not grow a
  monolithic spec.
- Do not skip validation silently.
- Query capability before optional external tool use.
- Run dependent commands sequentially. Do not put a durable write and its
  follow-up read or verification in `multi_tool_use.parallel`.
- Ask humans only for real ambiguity, high-risk direction, credentials, paid or
  destructive actions, or explicit approval gates.
- Leave durable records for the next agent.

## Answer Quality

Avoid abstract answers.

When explaining decisions, plans, risks, bugs, architecture, or trade-offs, use
concrete examples and step-by-step cause-and-effect reasoning.

Prefer this structure when useful:

1. What happens
2. Why it happens
3. Concrete example
4. Resulting impact
5. Recommended action

## Work Loop

1. Classify input type with `_harness/FEATURE_INTAKE.md`.
2. Run the risk checklist and choose `tiny`, `normal`, or `high-risk`.
3. Record intake when the CLI exists:
   `_harness/bin/harness-cli intake --type <type> --summary <text> --lane <lane>`.
4. Locate affected docs, stories, decisions, code, and tests.
5. Query proof matrix when the CLI exists.
6. Run `_harness/bin/harness-cli tool check` when the CLI exists.
7. Before optional external tools, run:
   `_harness/bin/harness-cli query tools --capability <capability> --status present`.
8. Implement the smallest safe slice for the lane.
9. Update product docs, story state, proof, decisions, templates, or backlog
   when the task changes them.
10. Validate for the lane.
11. Record a trace when the CLI exists.
12. Fix harness friction immediately or record backlog.

## Command Ordering

Parallelize only independent commands. Serialize every producer -> consumer
sequence.

Run these sequences in separate tool calls, in order:

- `harness-cli init` before any `harness-cli query ...`.
- `harness-cli story add/update/verify` before `harness-cli query matrix`.
- `harness-cli tool check` before `harness-cli query tools`.
- File edits before validation commands that read those files.
- Validation and durable story updates before `harness-cli trace`.

If violated, rerun the dependent read or validation sequentially and use only
the rerun result.

## Lanes

Tiny:

- Use for low-risk docs, copy, naming, narrow edits, or limited setup without
  schema, CRUD, auth, authorization, provider integration, or migrations.
- Record intake, patch directly, run quick checks, update changed docs.

Normal:

- Use for story-sized behavior with bounded blast radius.
- Create or update one story when behavior-bearing.
- Record proof with `story update`; store repeatable proof with `--verify` and
  run `story verify <id>` when available.
- Record a Standard trace.

High-risk:

- Use for security, data, scope, public contracts, multiple roles/platforms, or
  validation guarantees.
- Create a high-risk packet from `_harness/templates/high-risk-story/`.
- Read relevant decisions before implementation.
- Add a durable decision for meaningful behavior, architecture, authorization,
  data ownership, API shape, or validation changes.
- Record a Detailed trace.

Hard gates are high-risk unless the user explicitly narrows scope: auth,
authorization, data loss or migration, audit/security, external provider
behavior, or removing/weakening validation.

## Source Hierarchy

```text
Current user instruction
  -> docs/product/*
  -> docs/stories/*
  -> _harness/bin/harness-cli query matrix
  -> docs/decisions/* plus CLI decisions
  -> code and tests
  -> historical specs or examples
```

## Validation And Proof

Use the right checks or state the exact gap. For normal/high-risk story work:

```bash
_harness/bin/harness-cli story verify <story-id>
_harness/bin/harness-cli story update --id <story-id> --unit 1 --integration 1 --e2e 0 --platform 0 --evidence "<commands run>"
_harness/bin/harness-cli query matrix
```

Proof booleans use `1`/`0`, not `yes`/`no`.

## Trace And Final Response

Before the final response:

1. Re-check validation evidence.
2. Run `git status --short`.
3. Record a trace with `_harness/bin/harness-cli trace` when the CLI exists.
4. Confirm changed harness artifacts when relevant.
5. Confirm trace/friction status or name the gap.

Final response stays concise: changed surface, validation, durable records, and
remaining gap only.
