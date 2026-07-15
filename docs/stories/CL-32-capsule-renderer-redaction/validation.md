# CL-32 Validation

Prove safe path handling, redaction of token/password patterns and absolute
paths, collision refusal, valid checksum, no partial final file on staging
failure, and orphan reporting.

Evidence: packaged `memory capsule render` redacted secret keywords, the
following value and absolute path; `memory capsule check` accepted the valid
capsule and failed a deliberately corrupted checksum with exit `6`. The same
check reports staged `.tmp` files as orphan candidates. Existing final paths
are refused before staging.
