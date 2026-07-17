# 0025 Remove Legacy CLI Commands

Date: 2026-07-17

## Status

Accepted

## Context

The command-first rework made task lifecycle commands authoritative. The
execution plan explicitly classified `init`, `migrate`, `intake`,
`import brownfield`, standalone `trace`, `score-trace`, `score-context`, and
mutable `story` commands as compatibility-window commands. Keeping them in the
public tree lets agents bypass or misunderstand lifecycle ownership.

## Decision

Remove those compatibility commands and every shipped caller. Keep `story
check` because it validates tracked artifacts without acting as task lifecycle
state. `task start` creates the intake root; `proof run` records proof; and
`task finish` owns terminal closure and trace selection.

The user explicitly approved removal of all commands not used by the reworked
workflow on 2026-07-17.

## Consequences

Existing automation that invokes a removed command must migrate to the
command-first equivalent or stop invoking it. Source, packaged and installed
CLI help, tests and documentation must stay identical.
