# Run Classification Policy

## Purpose

Harness must classify work before creating a run. Large work must become an Epic instead of a normal run.

## Normal Run

Use a normal run only when the task is bounded, has one primary objective, and can be verified as one unit.

A normal run should usually affect a small number of related behaviours or files.

## Epic

Use an Epic when the task has multiple independently verifiable parts.

Epic signals include:

- multiple phases;
- multiple milestones;
- multiple user flows;
- multiple modules;
- uncertain or expanding scope;
- task text such as `phase`, `phase 1-4`, `part 1-4`, `core loop`, `full feature`, `complete playable`, `end-to-end`, `MVP`, `large task`, `long task`, or equivalent wording;
- cannot produce a clean verification result in one run;
- implementation would require several independently verifiable checkpoints.

Epic is mandatory when any Epic signal is present. Do not continue by creating a normal run.

## Child Run

A child run is a normal bounded run inside an Epic.

Each child run must have:

- one clear objective;
- its own planner brief;
- its own implementation contract;
- independent contract review;
- independent generator worklog;
- independent evaluator report;
- independent final summary;
- independent verification target.

A child run must not cover all Epic phases or all Epic acceptance criteria.

## Classification Decision

Before creating a normal run, answer:

1. Does the task mention multiple phases or parts?
2. Does the task require multiple behaviours or modules?
3. Can the task be verified cleanly in one run?
4. Can at least two independently verifiable child runs be identified?
5. Would one run create a broad or vague contract?

If the answer indicates broad scope, create an Epic.

## Invalid Oversized Normal Run

If a normal run is created but later found to be too large, mark it:

`SUPERSEDED_BY_EPIC`

Do not implement inside the invalid normal run.

Create an Epic and decompose the work into child runs.

## Required Status Block

```md
# Run Status

- Status: SUPERSEDED_BY_EPIC
- Reason: <why this task is too large for a normal run>
- Superseded by: <EPIC-ID>
- Next action: continue only through Epic child runs
```

## Example: Invalid Normal Run

Request:

> Develop playable DDM core loop phase 1-4

Wrong:

```txt
RUN-20260512-002-develop-playable-ddm-core-loop-phase-1-4
```

Reason:

- multiple phases;
- broad game loop;
- movement, dice, summon, combat, AI, and win condition likely need separate verification.

Correct:

```txt
EPIC-20260512-002-develop-playable-ddm-core-loop/
  runs/RUN-001-core-board-state-and-turn-shell/
  runs/RUN-002-dice-roll-summon-and-placement/
  runs/RUN-003-movement-terrain-and-combat/
  runs/RUN-004-ai-player-win-condition-and-playability/
```
