# CL-42 Overview

## Status

in_progress

`proof run` now invokes an executable with an argv vector from repository root.
It validates task and optional linked story, then appends pass or fail to
`proof_run` with exit code and current HEAD. `task status` exposes latest state
and whether its recorded HEAD still matches. Failed proof remains evidence; it
is not overwritten.
