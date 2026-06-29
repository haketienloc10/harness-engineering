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

From this repository checkout:

```bash
.agent-harness/install/install-harness.sh --directory /path/to/target --yes
```

Windows PowerShell:

```powershell
.\.agent-harness\install\install-harness.ps1 -Directory C:\path\to\target -Yes
```

Use merge mode when refreshing an existing target:

```bash
.agent-harness/install/install-harness.sh --directory /path/to/target --merge --yes
```

```powershell
.\.agent-harness\install\install-harness.ps1 -Directory C:\path\to\target -Merge -Yes
```

Use override only when replacing the target's harness surface is intentional.

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
