# CL-43 Overview

## Status

in_progress

`task finish --outcome completed` is now the sole code path to `completed`.
Tiny non-material tasks require an explicit no-capsule disposition; required
capsule tasks must supply a validated rendered capsule. Both paths require
owner, fresh proof, required context acknowledgement, matching trace and
`friction=none`.

High-risk completion additionally requires at least one approval record. A
repeat finish with the same capsule disposition and deterministic closure nonce
is idempotent and returns the existing completed state.
