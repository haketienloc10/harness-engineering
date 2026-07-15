# CL-43 Design

Finish recomputes context without mutation and refuses a stale manifest. It
checks owner, behavior/story link, proof HEAD and dirty freshness, matching
intake trace tier and unresolved friction before a SQLite immediate transaction
performs `in_progress → closing → completed`.

The tiny path writes an explicit no-capsule reason. Required-capsule tasks
validate a safe repository-relative capsule, schema, task ID and body checksum
before the transaction records its path/checksum. The valid source capsule is
copied to a same-filesystem staged path and revalidated; an SQLite `IMMEDIATE`
transaction records deterministic `closure_nonce`, moves the task to `closing`,
atomically renames the staged capsule, then records terminal state. If terminal
SQL fails after rename, SQLite rolls back to `in_progress`; a retry derives the
same nonce and safely reuses the final capsule. A leftover staged file is a
read-only `doctor` hard failure.

High-risk tasks require a recorded approval before preflight proceeds. A
completed task returns success only when the requested capsule disposition
matches its stored terminal record; mismatches fail closed.

Every completion gate returns one `StructuredErrorResult` containing stable
`ok`, `code`, `message`, `details` and `remediation` fields. Human and JSON
presentations render that same result. `TaskFinishGate` exits with code `5`;
ownership, durable-state and database failures retain their distinct exit-code
classes.

`doctor` reads terminal task records: a required capsule with missing, unsafe
or invalid file/checksum, a non-capsule terminal task without an omission
reason, or a leftover staged closure file makes durable state unhealthy.
