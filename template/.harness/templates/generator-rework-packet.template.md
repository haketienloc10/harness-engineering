# Generator Rework Packet

Role: Generator

## Reason

Evaluator or reviewer rejected the previous implementation.

## Failed Artifact

Path:
`<path-to-failed-report>`

## Failure Summary

<short summary copied from evaluator/reviewer decision>

## Required Fix

- <specific required fix>
- <specific required test or verification update>

## Allowed Inputs

- original task brief;
- approved contract;
- evaluator/reviewer report;
- directly relevant source files only.

## Forbidden Actions

- do not reinterpret product rules;
- do not edit unrelated files;
- do not modify Harness workflow rules;
- do not mark the run completed;
- do not write evaluator approval;
- do not broaden scope beyond the failed criterion.

## Required Output

- implementation diff summary;
- commands executed;
- test results;
- updated Generator rework report (`06-fix-report.md`).

## After Completion

Return control to the coordinator.

The coordinator must spawn Evaluator again.
