# Agent Harness

This directory is the agent runtime. Product records live in `docs/product/`,
`docs/stories/`, and `docs/decisions/`.

Humans provide intent, constraints, and high-risk approvals. Agents classify the
work, execute the lane, validate, and leave durable memory.

## Runtime Contract

Every task follows this shape:

```text
intent
  -> intake
  -> lane
  -> story when needed
  -> implementation or documented blocker
  -> validation
  -> trace
  -> friction fix or backlog
```

Every task may produce a product delta, a harness delta, or both. Product delta
means app code, tests, API shape, data model, or product docs. Harness delta
means docs, templates, validation expectations, backlog items, traces, or
decision records that make the next task safer.

Start from `AGENTS.md`. Use the detailed files for the detail they own:

| Need | Source |
| --- | --- |
| Intake type, risk flags, lanes | `_harness/FEATURE_INTAKE.md` |
| Context selection by phase and lane | `_harness/CONTEXT_RULES.md` |
| Architecture and boundary defaults | `_harness/ARCHITECTURE.md` |
| Optional external tool policy | `_harness/TOOL_REGISTRY.md` |
| Trace field depth and quality tiers | `_harness/TRACE_SPEC.md` |
| Fallback proof matrix | `_harness/TEST_MATRIX.md` |
| Drift and maturity references | `_harness/HARNESS_AUDIT.md`, `_harness/HARNESS_COMPONENTS.md`, `_harness/HARNESS_MATURITY.md` |
| Harness improvement loop | `_harness/IMPROVEMENT_PROTOCOL.md` |
| Story, decision, validation templates | `_harness/templates/` |

## Durable Layer

Use the repository-local CLI when it exists:

```bash
_harness/bin/harness-cli <command>
```

Initialize local state if needed:

```bash
_harness/bin/harness-cli init
```

Core commands:

```bash
_harness/bin/harness-cli intake --type <type> --summary <text> --lane <lane>
_harness/bin/harness-cli query matrix
_harness/bin/harness-cli tool check
_harness/bin/harness-cli story add --id <id> --title <text> --lane <lane> --verify "<command>"
_harness/bin/harness-cli story update --id <id> --unit 1 --integration 1 --e2e 0 --platform 0 --evidence "<commands>"
_harness/bin/harness-cli story verify <id>
_harness/bin/harness-cli story verify-all
_harness/bin/harness-cli decision add --id <id> --title <text> --doc docs/decisions/<file>.md
_harness/bin/harness-cli trace --summary <text> --agent <agent> --outcome completed
_harness/bin/harness-cli audit
_harness/bin/harness-cli propose
```

Operational data lives in ignored local `harness.db`; schema migrations live in
`_harness/scripts/schema/`. If the CLI is unavailable, use markdown artifacts
and record the missing CLI as harness friction.

## Command Ordering

The durable CLI is stateful. Serialize every producer -> consumer sequence.
Parallel tool calls are only for independent commands.

Do not combine dependent commands in one parallel batch:

- `init` -> `query ...`
- `story add/update/verify` -> `query matrix`
- `tool check` -> `query tools`
- file edits -> lint/test/typecheck
- proof recording -> `trace`

If violated, rerun the dependent read or validation sequentially and use only
the rerun result. Record friction only if the ordered rerun still fails.

## Source Hierarchy

When sources conflict:

```text
Current user instruction
  -> docs/product/*
  -> docs/stories/*
  -> _harness/bin/harness-cli query matrix
  -> docs/decisions/* plus CLI decisions
  -> code and tests
  -> historical specs or examples
```

User specs are input material. Living truth belongs in product docs, stories,
proof, and decisions.

## Spec Lifecycle

Treat a user-provided spec as input, not a permanent operating manual. Convert
accepted intent into product docs, story packets, validation expectations,
decisions, and durable proof.

Do not keep extending a monolithic spec after decomposition. Use scoped
initiative notes for large product areas that need multiple stories.

## Scope

Harness v0 includes:

- Agent entrypoint.
- Product documentation structure.
- Feature intake and risk lanes.
- Story, decision, and validation templates.
- Fallback test matrix.
- Durable SQLite layer and CLI.
- Trace, audit, maturity, and improvement references.

Harness v0 excludes:

- A project-specific `SPEC.md`.
- A locked application stack.
- App source scaffolding.
- Fake validation scripts or CI workflows.

Create stack folders, package scripts, and CI only when a selected story needs
them.

## Story Proof

Stories store proof in the durable layer. Run validation yourself, then record
the result:

```bash
_harness/bin/harness-cli story verify <story-id>
_harness/bin/harness-cli story update --id <story-id> --unit 1 --integration 1 --e2e 0 --platform 0 --evidence "<commands run>"
_harness/bin/harness-cli query matrix
```

Use numeric booleans: `1` means yes, `0` means no. Use
`query matrix --numeric` when copying proof values back into `story update`.

## External Tools

Run `tool check` at intake start and before optional external tools:

```bash
_harness/bin/harness-cli tool check
_harness/bin/harness-cli query tools --capability <capability> --status present
```

No provider registered means the capability is inactive and can be skipped.
Registered-but-missing providers are degraded proof and should be reported.

## Growth And Intervention

The harness grows from friction. If the task exposes confusion, stale docs,
missing proof, repeated manual work, or a recurring failure pattern, fix it in
scope or record backlog:

```bash
_harness/bin/harness-cli backlog add --title "<short name>" --pain "<what was hard>" --risk tiny
```

Use backlog predicted/actual outcomes for improvements that should change
agent behavior or validation results. Record human, reviewer, CI, or agent
corrections separately:

```bash
_harness/bin/harness-cli intervention add --trace <id> --type correction --description <text> --source human
```

## Decisions

High-risk work needs a durable decision when it changes behavior, architecture,
authorization, data ownership, API shape, audit/security, provider behavior, or
validation requirements:

1. Add `docs/decisions/NNNN-*.md` from `_harness/templates/decision.md`.
2. Record it:

```bash
_harness/bin/harness-cli decision add \
  --id 0008-auth-boundary \
  --title "Auth Boundary" \
  --doc docs/decisions/0008-auth-boundary.md
```

Trace `--decisions` is evidence, not a durable decision record.

## Change Policy

Agents may update routine story status, proof, traces, intake records, backlog,
validation notes, and small clarifications tied to the current task.

Ask before changing architecture direction, removing validation requirements,
changing source hierarchy, changing risk classification, or replacing the
feature workflow.

## Future Validation Ladder

Do not claim these commands pass until they exist and have been run:

```text
validate:quick
  format, lint, typecheck, unit tests, architecture check

test:integration
  backend, database, provider, or service checks

test:e2e
  user-visible end-to-end flows

test:platform
  shell, mobile, desktop, or deployment smoke checks

test:release
  full suite, log checks, and performance smoke
```

## Done Definition

A task is done only when:

- The requested change is completed or the blocker is documented.
- Relevant docs, stories, proof, decisions, and templates remain current.
- Available validation was run, or the exact gap is stated.
- A trace was recorded when the CLI exists.
- Harness friction was fixed or recorded.
- The final response says what changed and what was not attempted.
