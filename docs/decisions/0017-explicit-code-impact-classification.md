# 0017 Explicit Code-Impact Classification

Date: 2026-07-14

## Status

Accepted

## Decision

The command-first lifecycle accepts behavior/code impact as explicit structured
input. It does not infer that signal from a free-text summary. Until `task
start` owns that input, the shadow classifier uses only supplied risk flags and
hard gates; one flag therefore remains `tiny`.

## Consequences

The current Markdown instruction that an agent may choose `tiny` or `normal`
for zero or one flag based on code impact remains authoritative during shadow
mode. CL-41 must expose a validated behavior-bearing/code-impact input before
`workflow.toml` becomes the authority for lifecycle classification.
