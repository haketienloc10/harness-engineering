# 0015 Trace and Capsule Privacy

Date: 2026-07-14

## Status

Accepted

## Decision

Tracked capsules contain only structured summaries, hashes, and repo-relative
paths. They exclude secrets, raw prompts, raw command logs, and machine
absolute paths. Raw proof output lives in ignored, size-limited evidence paths;
capsule rendering redacts configured secret patterns and rejects unsafe fields.

## Consequences

Privacy/redaction validation is a task-finish gate whenever a capsule is
required.
