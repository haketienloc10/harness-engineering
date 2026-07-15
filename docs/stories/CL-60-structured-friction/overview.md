# CL-60 Overview

## Status

in_progress

Migration `009-structured-friction.sql`, ADR 0021 and `friction
add|resolve|query` establish the durable lifecycle. `task finish` rejects a
linked material record until it has a terminal observation outcome.

Source implementation is present, but the 2026-07-15 audit found that the
packaged `_harness/bin/harness-cli` command surface does not include the
`friction` commands. CL-60 remains open until the packaged binary, tracked
command manifest and installer parity proof agree at the same HEAD.
