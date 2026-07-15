# CL-42 Design

The command uses `Command::new(executable).args(argv)`, never joins input into a
shell string. Doctor health is required before task/proof reads and writes.
After execution, the service stores the executable, JSON argv, state, exit code,
completion timestamp and `git rev-parse HEAD`. It also stores a versioned JSON
summary containing a SHA-256 fingerprint of tracked diff plus untracked paths
and bytes; this uses the existing `summary` field to preserve canonical schema
lineage. `proof query` exposes append-only rows, while `task status` compares
the latest row's HEAD and dirty fingerprint to the current worktree.

Branch, raw-output hashes and artifact-scoped freshness remain deferred. The
finish gate must treat the absence of those later provenance fields explicitly,
not as a pass.
