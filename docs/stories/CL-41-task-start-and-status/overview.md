# CL-41 Overview

## Status

in_progress

`task start` now requires `--behavior-bearing yes|no`; it intentionally does
not infer code impact from free-text summaries (ADR 0017). Without `--lane`, it
derives lane from workflow flags; a different explicit lane requires
`--lane-reason` and cannot lower a policy classification. It performs doctor
preflight and writes intake, task and an optional existing primary-story link
in one transaction. A behavior-bearing task is refused when the selected
policy lane requires a story, and a different owner cannot concurrently open
the same primary story. A started task enters `in_progress`.

The service stores the policy's work-phase context manifest and checksum at
start. `task context acknowledge` accepts only a stored must/should-read path;
`task approve` accepts only a workflow-declared approval gate and stores source,
evidence and optional scope. `task status` reports transition, context and
approval summaries.

`task refresh --id` recomputes the effective work context from stored lane,
flags and task-story links. A context delta exits without mutation until the
caller explicitly supplies `--accept`.

Owned tasks require matching `--owner` for block/resume/abandon. `task handoff`
records source/evidence/scope and atomically assigns a new owner (ADR 0019).

`task link-story` explicitly adds a secondary story or promotes one primary;
promotion demotes the former primary in the same transaction.
