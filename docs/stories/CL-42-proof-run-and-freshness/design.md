# CL-42 Design

The command uses `Command::new(executable).args(argv)`, never joins input into a
shell string. Doctor health is required before task/proof reads and writes.
Migration 011 extends the canonical `proof_run` record with story, cwd, branch,
dirty fingerprint, CLI/platform, command digest, stdout/stderr and optional
artifact provenance. Existing rows remain readable; absent branch or output
provenance is stale rather than an implicit pass.

Raw stdout/stderr is stored under ignored `.harness-evidence/proofs/` paths.
Each stream is capped at 1 MiB and its retained bytes are SHA-256 hashed. The
tracked record stores only repo-relative paths and hashes. `--artifact` accepts
one safe repo-relative file; a missing or escaping artifact makes the run fail,
and status compares its current hash to the recorded hash.

The dirty fingerprint is SHA-256 over domain-separated `git diff --binary
--no-ext-diff HEAD`, followed by `git ls-files --others --exclude-standard -z`
entries in Git order. Each untracked entry contributes its raw path, a NUL
separator, byte length and file bytes. Ignored evidence output is therefore not
part of the worktree fingerprint. `task status` independently derives HEAD,
branch, dirty, output and artifact freshness.

`query matrix` uses the latest structured row for each story/layer and falls
back to legacy numeric columns only when no structured row exists. New direct
proof-boolean writes through `story update` are rejected; historical imported
booleans remain read-compatible.
