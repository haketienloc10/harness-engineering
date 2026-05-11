# Coding Conventions

## General

- Prefer simple, readable code.
- Do not over-engineer.
- Do not introduce new dependencies unless necessary.
- Keep changes scoped to the approved contract.
- Avoid unrelated refactors.
- Preserve existing project conventions.

## Before Editing

Before editing any file:
- read the file first
- inspect nearby code
- search for callers/usages before modifying existing functions
- understand existing conventions

## Naming

Use existing naming conventions in the repository.

If no convention exists:
- use clear names
- avoid abbreviations unless domain-standard
- keep method names action-oriented
- keep class/component names purpose-oriented

## Error Handling

- Do not swallow errors silently.
- Prefer explicit error paths.
- Keep user-facing errors clear.
- Keep internal logs useful but not noisy.

## Frontend Rules

- Keep components small enough to read.
- Prefer explicit props and simple local state.
- Avoid unnecessary abstractions.
- Keep form validation visible to the user.
- Keep UI states clear: empty, loading, error, success when applicable.

## Scope Control

The generator must not:
- rewrite unrelated files
- introduce broad architectural changes without contract approval
- add speculative features
- change public API without documenting it in the contract
