# CL-43 Exec Plan: Task Finish and Crash Recovery

## Goal

Make `task finish` the only command that can set `completed`, with fail-closed
proof, context, trace, ownership and capsule gates.

## Scope

The implementation owns transactional tiny and required-capsule completion,
normal/high-risk gates, deterministic nonce idempotency and recovery after a
post-rename terminal-SQL rollback.

## Risk Classification

Risk flags: durable state, validation guarantees, privacy/capsule behavior.
Hard gate: weakening validation is prohibited.

## Stop Conditions

Do not claim normal/high-risk completion until staged capsule rename, DB
transaction rollback and recovery retry are proven.
