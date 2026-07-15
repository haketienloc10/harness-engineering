# CL-32 Design

`memory capsule render` accepts structured metadata rather than free-form raw
logs. It writes to `docs/tasks/YYYY/MM/TASK-id-slug.md` only after rendering,
redacting and validating content/checksum in a same-directory temporary file.
Existing final paths are rejected. Orphan staged files are reported by a
read-only command.
