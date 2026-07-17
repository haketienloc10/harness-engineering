# CL-60 Overview

## Status

completed

Migration `009-structured-friction.sql`, ADR 0021 and `friction
add|resolve|query` establish the durable lifecycle. `task finish` rejects a
linked material record until it has a terminal observation outcome.

The 2026-07-15 completion rerun confirms that the source implementation,
packaged `_harness/bin/harness-cli`, tracked command manifest and installer
surface expose the same `friction` commands at HEAD `0df8291`.
