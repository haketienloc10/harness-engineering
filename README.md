# Agent-First Harness Payload

This repository is an installable framework for coding agents. Its primary
consumer is the agent running inside a target repository. Humans provide intent,
constraints, and high-risk approvals.

## Install Surface

Install these paths into a target repo:

- `AGENTS.md`: mandatory root entrypoint for agents.
- `_harness/`: agent policy, templates, schema migrations, and the local CLI.
- `docs/product/`: living product contract derived from accepted input.
- `docs/stories/`: story packets and progress evidence.
- `docs/decisions/`: durable decision records.
- `_harness/scripts/schema/`: durable-layer database migrations used by the CLI.
- `_harness/bin/harness-cli`: repository-local CLI command for agents.
- `.gitignore`: durable local state rules for `harness.db` and CLI binaries.

Avoid installing framework policy into `docs/`. In target repositories, `docs/`
belongs to the product; Harness-managed product truth, story packets, and
decision records live there.

## Agent Runtime Contract

An agent starts with `AGENTS.md`, then reads `_harness/` by lane and task
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
_harness/bin/harness-cli init
_harness/bin/harness-cli query matrix
```

## Install Commands

Install into the current repository:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/install.sh?$(date +%s)" | bash
```

Install into another directory:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/install.sh?$(date +%s)" | HARNESS_LITE_TARGET_DIR=/path/to/target bash
```

From a local checkout, run the same installer:

```bash
HARNESS_LITE_TARGET_DIR=/path/to/target ./install.sh
```

The installer downloads the source archive from
`haketienloc10/harness-engineering`, installs the shared scaffold files, updates
`_harness/` files, preserves existing target files outside `_harness/`, embeds a
Harness block into the target `AGENTS.md`, filters source-only artifacts, and
writes `_harness/.harness-manifest`.

The root installer does not build the CLI. In this source repository, refresh the
repository-local CLI binary from the Rust source with:

```bash
./install-harness-cli.sh
```

## Payload Structure

```text
target-repo/
  AGENTS.md
  _harness/
    HARNESS.md
    FEATURE_INTAKE.md
    CONTEXT_RULES.md
    ARCHITECTURE.md
    TOOL_REGISTRY.md
    TRACE_SPEC.md
    TEST_MATRIX.md
    templates/
    scripts/schema/
    bin/harness-cli      # repository-local Rust CLI binary
  docs/
    product/
    stories/
    decisions/
  harness.db            # local ignored database
```

## Design Rule

This payload is allowed to be strict. It exists to make agents reliable in
unfamiliar repos. Human-friendly explanation is secondary to clear operating
commands, durable records, and validation proof.
