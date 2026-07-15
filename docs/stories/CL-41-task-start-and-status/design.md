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
different agent. This is a conservative owner-conflict foundation; leases and
handoff approvals remain deferred.

The start transaction compiles `workflow.toml` work context from lane, flags
and linked story, then persists its JSON plus checksum. Acknowledgement reads
that exact manifest and refuses an unlisted path; it adds a de-duplicated
attestation row. Approval validates its gate against the effective workflow
policy before storing the supplied source, evidence and scope.

Refresh derives a new manifest from stored semantic inputs and reports a sorted
path delta. It only updates the stored JSON/checksum with `--accept`, so it
cannot silently reduce an earlier gate.

Transitions read the stored owner inside their immediate transaction. A mismatch
fails before state change. Handoff requires different source/target owners,
writes the owner and a `handoff` approval record in the same transaction.

Story linking uses the existing link table. Only one row can carry `primary` at
a time: promotion demotes other primary rows before an upsert, while secondary
links are retained.

Only declared domain transitions are accepted. `block`, `resume` and
`abandon` use the same transition service. No command can transition to
`completed`; that remains a gated CL-43 concern.
