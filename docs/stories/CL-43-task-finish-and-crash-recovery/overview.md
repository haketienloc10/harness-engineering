# CL-43 Overview

## Status

completed

`task finish --outcome completed` is now the sole code path to `completed`.
Tiny non-material tasks require an explicit no-capsule disposition; required
capsule tasks must supply a validated rendered capsule. Both paths require
owner, fresh proof, required context acknowledgement, matching trace and
`friction=none`.

High-risk completion additionally requires at least one approval record. A
repeat finish with the same capsule disposition and deterministic closure nonce
is idempotent and returns the existing completed state.

The final Phase 4 source/packaged failure matrix, installer state-safety,
source/packaged workflow parity, memory validation and strict doctor gates all
passed before this status moved to `completed`.
