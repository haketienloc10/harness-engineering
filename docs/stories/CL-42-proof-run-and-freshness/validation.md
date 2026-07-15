# CL-42 Validation

Unit proof executes `git --version` as a passing structured proof and an invalid
Git subcommand as a failing proof; both rows are retained for the same task.
It also covers task/story-link validation through the lifecycle fixture.
Task status asserts the latest failed row, run count and matching HEAD/dirty
fingerprint result, then verifies an untracked file makes dirty freshness fail.

Required before completion: packaged query proof, branch/output provenance and
artifact-scoped freshness cases.

Packaged proof: a temporary `HARNESS_DB` completed `init`, linked a normal
task/story, ran `proof run -- git --version`, then returned a versioned summary
with dirty fingerprint from `proof query --json`; `task status --json` reported
both `head_fresh:true` and `dirty_fresh:true`.
