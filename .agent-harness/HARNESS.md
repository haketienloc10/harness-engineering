# Agent Harness

This directory defines how agents operate inside the target repository. Humans
provide intent, constraints, and high-risk approvals; agents classify, execute,
validate, and maintain memory.

The app is what users touch. The harness is what agents touch.

## Mental Model

```text
------------------+
| Human intent    |
+------------------+
         |
         v
+------------------+
| Feature intake   |
+------------------+
         |
         v
+------------------+
| Story packet     |
+------------------+
         |
         v
+------------------+
| Agent work loop  |
+------------------+
         |
         v
+------------------+
| Product delta    |
+------------------+
         |
         v
+------------------+
| Validation proof |
+------------------+
         |
         v
+------------------+
| Harness delta    |
+------------------+
         |
         v
+------------------+
| Next intent      |
+------------------+
```

Every task has two possible outputs:

1. Product delta: app code, tests, API shape, data model, or product docs.
2. Harness delta: docs, templates, validation expectations, backlog items, or
   decision records that make the next task easier.

## Harness v0 Scope

Harness v0 includes:

- Agent entrypoint.
- Empty product documentation structure.
- Feature intake and risk lanes.
- Story templates.
- Decision log template.
- Validation report template.
- Test matrix placeholder.
- Harness growth backlog through the durable CLI.
- Durable layer: SQLite database and CLI for operational records.

Harness v0 deliberately excludes:

- A project-specific `SPEC.md`.
- Pre-sliced product domains.
- A locked application stack.
- App source scaffolding.
- Package scripts.
- Test runner config.
- CI workflows.

Those should arrive only when a selected story needs them.

## Durable Layer

Policy documents describe how to work. The durable layer stores what happened.

Operational data — intake classifications, story status, decision outcomes,
backlog items, and execution traces — lives in a SQLite database (`harness.db`)
managed by the Rust Harness CLI at `.agent-harness/bin/harness-cli`. Agents must
use that binary for Harness work when it exists. Humans may inspect it, but
routine task state belongs to the agent. The database is local to each project
instance and `.gitignore`d. The schema is version-controlled under
`.agent-harness/scripts/schema/`.

This separation keeps policy docs stable while giving agents a structured,
queryable record of operational state. It also prepares the harness for future
observability and automated evolution without adding more markdown files.

Initialize the database if it does not exist:

```bash
.agent-harness/bin/harness-cli init
```

Common commands:

```bash
.agent-harness/bin/harness-cli intake  --type <type> --summary <text> --lane <lane>
.agent-harness/bin/harness-cli story   add --id <id> --title <text> --lane <lane>
.agent-harness/bin/harness-cli story   update --id <id> --status <status>
.agent-harness/bin/harness-cli story   update --id <id> --unit 1 --integration 1 --e2e 0 --platform 0
.agent-harness/bin/harness-cli story   verify <id>
.agent-harness/bin/harness-cli story   verify-all
.agent-harness/bin/harness-cli decision add --id <id> --title <text> --doc .agent-harness/decisions/<file>.md
.agent-harness/bin/harness-cli trace   --summary <text> --outcome <outcome>
.agent-harness/bin/harness-cli score-trace
.agent-harness/bin/harness-cli score-context <trace-id>
.agent-harness/bin/harness-cli audit
.agent-harness/bin/harness-cli propose
.agent-harness/bin/harness-cli query   matrix
.agent-harness/bin/harness-cli query   matrix --numeric
.agent-harness/bin/harness-cli query   backlog
.agent-harness/bin/harness-cli query   tools --summary
.agent-harness/bin/harness-cli query   interventions
.agent-harness/bin/harness-cli query   stats
.agent-harness/bin/harness-cli --version
```

## Source Hierarchy

```text
User-provided spec or prompt
  input material for first buildout or future changes

.agent-harness/product/*
  current product contract derived from accepted input

.agent-harness/stories/*
  story-sized work packets and historical evidence

.agent-harness/bin/harness-cli query matrix
  behavior-to-proof control panel backed by the durable layer

.agent-harness/decisions/*
  why the contract changed
```

Before implementation, product docs describe intent. After implementation,
product docs plus executable tests become the living contract.

## Spec Lifecycle

Harness v0 starts without a tracked project spec. When the human provides a
specification, treat it as input material, not as a permanent operating manual.
Use it to populate product docs, story packets, architecture decisions, and
validation expectations during the first buildout.

After the specification has been decomposed, do not keep extending it as the
living product plan. Ongoing work should update the smaller product docs,
stories, durable proof records, and decision records.

Ongoing work should enter the harness as one of these input types:

- New spec: a project specification that needs to become product docs and
  initial story candidates.
- Spec slice: a selected behavior from the provided spec.
- Change request: a bounded behavior change, bug fix, or product refinement.
- New initiative: a larger product area that needs multiple stories.
- Maintenance request: dependency, architecture, performance, security, or
  operational work.
- Harness improvement: a process, template, proof, or agent-instruction change.

The spec-to-work loop is:

```text
human intent or supplied spec
  -> classify input type
  -> update or create product contract
  -> create story packet or initiative notes when needed
  -> define validation proof
  -> implement or document the blocker
  -> update product docs, stories, durable proof records, and decisions
  -> capture harness friction
```

Large product areas should use scoped initiative notes instead of a second
monolithic specification. An initiative should explain the goal, affected
product docs, candidate stories, validation shape, open decisions, and exit
criteria. If initiative work becomes a repeated pattern, add a template or
record the proposal with `.agent-harness/bin/harness-cli backlog add`.

## Growth Rule

The harness grows from friction.

When an agent is confused, repeats manual reasoning, needs a new validation
command, discovers a missing rule, or sees a recurring failure pattern, it must
either improve the harness directly or record the friction:

```bash
.agent-harness/bin/harness-cli backlog add --title "<short name>" --pain "<what was hard>"
```

Use the backlog outcome loop for improvements that are expected to change agent
behavior or validation results:

1. When creating the backlog item, fill `--predicted` with the measurable impact
   expected from the improvement.
2. When closing the item, fill `--outcome` with the actual measured result or
   review evidence.
3. Use `.agent-harness/bin/harness-cli query backlog --open` to review proposed
   and accepted items, and
   `.agent-harness/bin/harness-cli query backlog --closed` to compare
   predictions with outcomes after implementation.

The `harness_friction` field on traces also captures per-task friction so
patterns can be queried later:

```bash
.agent-harness/bin/harness-cli query friction
```

Backlog risk uses the same lane vocabulary as intake and stories: `tiny`,
`normal`, or `high-risk`. Use `--risk tiny` for low-risk follow-up items; `low`
is not a valid lane.

## Task Loop

For every task:

1. Classify the request with `.agent-harness/FEATURE_INTAKE.md`.
2. Record the classification with `.agent-harness/bin/harness-cli intake`.
3. Locate the affected product docs and story files.
4. Check proof status with `.agent-harness/bin/harness-cli query matrix`.
5. Work only inside the selected lane: tiny, normal, or high-risk.
6. Before finishing, ask whether product truth, validation expectations,
   architecture rules, repeated failure patterns, or next-agent instructions
   changed.
7. Record a trace with `.agent-harness/bin/harness-cli trace`, using
   `.agent-harness/TRACE_SPEC.md` for the expected trace tier and field depth.
8. Review the trace score printed by `.agent-harness/bin/harness-cli trace`; use
   `.agent-harness/bin/harness-cli score-trace --id <id>` only when re-checking
   a specific historical trace.
9. If harness friction was found, either fix it directly or record it with
   `.agent-harness/bin/harness-cli backlog add`.

## Story Verification

Stories may carry a mechanical proof command:

```bash
.agent-harness/bin/harness-cli story add --id US-012 --title "Story verification" --lane normal --verify "cargo test --workspace"
.agent-harness/bin/harness-cli story update --id US-012 --verify "cargo test --workspace"
.agent-harness/bin/harness-cli story verify US-012
```

`story verify` runs the command from the repository root, records
`last_verified_at` and `last_verified_result`, and exits 0 on pass or 1 on fail.
When `trace --story <id>` links to a story whose verification command has never
passed, the trace still records but prints an advisory warning before close.

Use `story verify-all` before merges, release claims, or broad completion
claims. It runs every configured story verification command, prints one result
per story, skips stories without `verify_command`, and exits 1 if any configured
story fails.

`story verify` accepts only the story id. Configure the command with
`story add --verify` or `story update --verify`. Record proof booleans with
`story update`, using numeric values: `1` means yes and `0` means no. The Rust
CLI rejects text values such as `yes` and `no`.

Use `.agent-harness/bin/harness-cli query matrix --numeric` when copying proof
values back into `story update`. The default matrix output is human-readable
`yes`/`no`; the numeric output mirrors CLI input.

## Evolution Commands

Tool discovery:

```bash
.agent-harness/bin/harness-cli query tools --summary
.agent-harness/bin/harness-cli query tools --json
.agent-harness/bin/harness-cli tool register --name <name> --command <cmd> --description <text> --responsibility Verification
```

Context and drift checks:

```bash
.agent-harness/bin/harness-cli score-context <trace-id>
.agent-harness/bin/harness-cli audit
```

`score-context` is advisory; it reports context-rule coverage without changing
the trace. `audit` reports drift categories and an entropy score.

Interventions are separate from traces:

```bash
.agent-harness/bin/harness-cli intervention add --trace <id> --type correction --description <text> --source human
.agent-harness/bin/harness-cli query interventions --story <story-id>
```

Record an intervention when a human, reviewer, CI system, or another agent
corrects, overrides, escalates, or approves work.

Improvement proposals:

```bash
.agent-harness/bin/harness-cli propose
.agent-harness/bin/harness-cli propose --commit
```

`propose` prints deterministic proposals from repeated friction, interventions,
and audit drift. `--commit` creates proposed backlog items only; it does not
edit policy docs or approve the proposal.

## Decision Records

High-risk work needs durable decisions when it changes behavior or architecture.
For auth, authorization, data ownership, API shape, audit/security, or
validation changes, record the decision in both places:

1. Add a markdown file under `.agent-harness/decisions/` from
   `.agent-harness/templates/decision.md`.
2. Add or refresh the durable record:

```bash
.agent-harness/bin/harness-cli decision add \
  --id 0008-auth-boundary \
  --title "Auth Boundary" \
  --doc .agent-harness/decisions/0008-auth-boundary.md \
  --notes "Accepted during T4 authentication work."
```

The trace `--decisions` field is useful evidence, but it is not the decision
log. Do not treat decision text in a trace as satisfying the durable decision
record requirement.

## Harness Change Policy

Agents may update directly:

- Story status and evidence via `.agent-harness/bin/harness-cli story update`.
- Test matrix rows via `.agent-harness/bin/harness-cli story add` and
  `.agent-harness/bin/harness-cli story update`.
- Links from story packets to product docs.
- Validation notes and reports.
- Small clarifications tied to the current task.
- Intake records, traces, and backlog items via
  `.agent-harness/bin/harness-cli`.

Agents should ask for human confirmation before:

- Changing architecture direction.
- Removing validation requirements.
- Changing the source-of-truth hierarchy.
- Changing risk classification rules.
- Replacing the feature workflow.

## Done Definition

A task is done only when:

- The requested change is completed or the blocker is documented.
- Relevant docs, stories, and test matrix entries remain current.
- Validation commands were run when they exist.
- A trace has been recorded with `.agent-harness/bin/harness-cli trace`.
- Missing harness capabilities were recorded with
  `.agent-harness/bin/harness-cli backlog add`.
- The final response says what changed and what was not attempted.

## Future Validation Ladder

No validation scripts exist yet. When implementation begins, the expected ladder
is:

```text
validate:quick
  format, lint, typecheck, unit tests, architecture check

test:integration
  backend, database, provider, or service checks as the stack requires

test:e2e
  user-visible end-to-end flows

test:platform
  shell, mobile, desktop, or deployment smoke checks as the stack requires

test:release
  full suite, log checks, and performance smoke
```

Agents must not claim these commands pass until they exist and have been run.
