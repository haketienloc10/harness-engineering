# CL-41 Exec Plan: Task Start and Status

## Goal

Create an explicit lifecycle root through `task start` and expose safe status
and non-completion lifecycle operations.

## Risk

High-risk durable records, task ownership and behavior-bearing classification.

## Scope

`task start`, `status`, `block`, `resume`, `abandon`, `approve`, and context
acknowledgement; atomic intake/task/primary-story creation; explicit
behavior-bearing input; doctor preflight; persisted policy manifest.

## Deferred

Lease expiry/session identity remain deferred.
`completed` remains owned solely by CL-43 `task finish`.
