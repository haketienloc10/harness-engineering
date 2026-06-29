# Agent-First Harness Payload

This repository is an installable framework for coding agents. Its primary
consumer is the agent running inside a target repository. Humans provide intent,
constraints, and high-risk approvals.

## Install Surface

Install these paths into a target repo:

- `AGENTS.md`: mandatory root entrypoint for agents.
- `.agent-harness/`: agent policy, templates, product memory, story packets, and
  decisions.
- `.agent-harness/scripts/schema/`: durable-layer database migrations used by
  the wrapped CLI.
- `.agent-harness/bin/harness-cli`: tracked wrapper for agents.
- `.agent-harness/bin/harness-cli.bin`: local downloaded binary, ignored by git.
- `.gitignore`: durable local state rules for `harness.db` and CLI binaries.

Avoid installing framework policy into `docs/`. In target repositories, `docs/`
belongs to the product unless the product chooses otherwise.

## Agent Runtime Contract

An agent starts with `AGENTS.md`, then reads `.agent-harness/` by lane and task
phase. The human is not expected to classify risk, maintain proof records, or
choose routine execution steps.

Required loop:

```text
intent
  -> intake
  -> lane
  -> story when needed
  -> implementation
  -> validation
  -> trace
  -> friction or backlog
```

Initialize local durable state after install:

```bash
.agent-harness/bin/harness-cli init
.agent-harness/bin/harness-cli query matrix
```

## Install Commands

Install into the current repository:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/install.sh?$(date +%s)" | bash
```

Install into another directory:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/install.sh?$(date +%s)" | bash -s -- /path/to/target
```

From a local checkout, run the same installer:

```bash
./install.sh /path/to/target
```

The installer backs up existing `AGENTS.md` and `.agent-harness/` under
`.harness-backup/`, installs the current payload, appends local Harness ignore
rules, and installs the CLI by downloading a compatible release binary or
building from `crates/harness-cli` when source and `cargo` are available.

## Payload Structure

```text
target-repo/
  AGENTS.md
  .agent-harness/
    HARNESS.md
    FEATURE_INTAKE.md
    CONTEXT_RULES.md
    ARCHITECTURE.md
    TOOL_REGISTRY.md
    TRACE_SPEC.md
    TEST_MATRIX.md
    product/
    stories/
    decisions/
    templates/
    scripts/schema/
    bin/harness-cli      # tracked wrapper
    bin/harness-cli.bin  # local ignored binary
  harness.db            # local ignored database
```

## Design Rule

This payload is allowed to be strict. It exists to make agents reliable in
unfamiliar repos. Human-friendly explanation is secondary to clear operating
commands, durable records, and validation proof.
