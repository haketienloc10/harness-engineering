# 0012 Task Lifecycle and Closure Invariants

Date: 2026-07-14

## Status

Accepted

## Decision

`task start` creates the lifecycle root and `task finish` exclusively owns the
transition to `completed`. Required context, story, approval, proof, friction,
trace, and capsule gates fail closed. Closure uses a staged capsule plus SQLite
transaction protocol and supports idempotent recovery through a closure nonce.

## Consequences

Standalone traces cannot close tasks. The application layer validates
transitions and the schema independently protects valid states.
