# CL-41 Overview

## Status

completed

`task start` requires `--behavior-bearing yes|no`; it intentionally does
not infer code impact from free-text summaries (ADR 0017). Without `--lane`, it
derives lane from workflow flags; a different explicit lane requires
`--lane-reason` and cannot lower a policy classification. It performs doctor
preflight and writes intake, task and an optional existing primary-story link
in one transaction. A behavior-bearing task is refused when the selected
policy lane requires a story, and a different owner cannot concurrently open
the same primary story. A started task enters `in_progress`.

Owned tasks require an explicit `--session` paired with `--owner`. They receive
a renewable lease (default 3,600 seconds; accepted range 60..86,400) stored by
migration `010-task-session-lease.sql`. One active lifecycle root is allowed
per session. An active primary-story root remains exclusive even after its
worktree lease expires, so a different agent must use the recorded handoff path
instead of creating a parallel takeover. A live lease also prevents a
different session from claiming the same worktree.

The service stores the policy's work-phase context manifest and checksum at
start. `task context acknowledge` accepts only a stored must/should-read path;
`task approve` accepts only a workflow-declared approval gate and stores source,
evidence and optional scope. `task status` reports transition, context and
approval summaries together with session, worktree, lease expiry and derived
lease state.

`task refresh --id` recomputes the effective work context from stored lane,
flags and task-story links. A context delta exits without mutation until the
caller explicitly supplies `--accept`.

Owned tasks require matching `--owner` and `--session` for
block/resume/abandon. Blocking releases the worktree lease. `task resume`
reacquires a blocked or expired lease and also renews an `in_progress` retry,
including adoption of a new session ID by the same owner after release/expiry.
It fails if the old lease is still active or another live session has claimed
the worktree. `task handoff` records source/evidence/scope and atomically
assigns a distinct owner/session with a fresh lease (ADR 0019).

`task link-story` explicitly adds a secondary story or promotes one primary;
promotion demotes the former primary in the same transaction.
