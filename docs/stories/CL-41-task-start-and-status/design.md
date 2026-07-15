# CL-41 Design

The write path first requires a `HEALTHY` doctor result. A SQLite immediate
transaction allocates a task ID, inserts the intake, inserts the task root and
optionally links the existing primary story. Any failed story link rolls the
whole transaction back.

Before allocation, the service reads the validated workflow policy to derive
lane from flags unless the caller supplies a documented non-lowering override;
its story rule then determines whether the explicit behavior-bearing value
requires a story.
Within the transaction it rejects an active primary-story task owned by a
different agent or session.

Migration 010 adds nullable `session_id` and `lease_expires_at` columns for
legacy compatibility, plus an insert trigger that rejects partial identity.
New owned tasks require `owner`, `session_id` and lease expiry together;
ownerless tasks keep all three values null. The application accepts leases from
60 through 86,400 seconds and defaults to 3,600 seconds.

An immediate start transaction rejects an existing active task in the same
session, an existing active primary-story task regardless of lease expiry, and
a different live session in the same worktree. Story exclusivity is durable
until handoff or terminal disposition; lease expiry only releases the
worktree-level execution claim. `block` releases that claim, while `resume`
atomically rechecks session/story/worktree conflicts and renews it. Re-running
`resume` for an `in_progress` task is the explicit retry path. The same owner
may adopt a new session ID only after the stored lease is released or expired;
an active lease still rejects a mismatched session.

The start transaction compiles `workflow.toml` work context from lane, flags
and linked story, then persists its JSON plus checksum. Acknowledgement reads
that exact manifest and refuses an unlisted path; it adds a de-duplicated
attestation row. Approval validates its gate against the effective workflow
policy before storing the supplied source, evidence and scope.

Refresh derives a new manifest from stored semantic inputs and reports a sorted
path delta. It only updates the stored JSON/checksum with `--accept`, so it
cannot silently reduce an earlier gate.

Transitions read the stored owner and session inside their immediate
transaction. A mismatch fails before state change. Handoff requires distinct
source/target owners and sessions, writes both target identities with a fresh
lease, and records a `handoff` approval in the same transaction.

Story linking uses the existing link table. Only one row can carry `primary` at
a time: promotion demotes other primary rows before an upsert, while secondary
links are retained.

Only declared domain transitions are accepted. `block`, `resume` and
`abandon` use the same transition service. No command can transition to
`completed`; that remains a gated CL-43 concern.
