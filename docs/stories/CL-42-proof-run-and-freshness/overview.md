# CL-42 Overview

## Status

completed

`proof run` invokes an executable with an argv vector from repository root. It
validates task and optional linked story, then appends pass or fail with command,
HEAD, branch, dirty-worktree, runtime, bounded output and optional artifact
provenance. `task status` derives freshness for HEAD, branch, dirty fingerprint,
output files and artifact independently. Failed proof remains evidence; it is
not overwritten, and the matrix derives story-layer state from structured runs.
