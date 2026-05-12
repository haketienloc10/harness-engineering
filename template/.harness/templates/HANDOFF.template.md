# Harness Role Handoff

## Current Run

`<RUN-DIR>`

## Parent Epic

`<PARENT-EPIC>`

## Current State

`<STATE>`

## Completed Role

`<COMPLETED-ROLE>`

## Next Required Role

`<NEXT-ROLE>`

## Required Files To Read

<REQUIRED-FILES>

## Forbidden Actions

<FORBIDDEN-ACTIONS>

## Required Output Artifact

`<REQUIRED-ARTIFACT>`

## Decision Criteria

<DECISION-CRITERIA>

## Copy-Paste Prompt For New Codex CLI Session

```txt
You are continuing a Harness lifecycle run in an independent Codex CLI session.

Run directory:
<RUN-DIR>

Parent epic:
<PARENT-EPIC>

Current lifecycle state:
<STATE>

Completed role:
<COMPLETED-ROLE>

You are the next required role executor:
<NEXT-ROLE>

Read these files before acting:
<REQUIRED-FILES>

Forbidden actions:
<FORBIDDEN-ACTIONS>

Produce this required output artifact:
<REQUIRED-ARTIFACT>

Decision criteria:
<DECISION-CRITERIA>

Use run.yaml as the authoritative workflow state. Do not skip lifecycle states. Do not claim completion unless the required evaluator evidence exists.
```
