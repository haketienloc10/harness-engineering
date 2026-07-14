# CL-21 Context Compiler

Status: in progress

`workflow context` compiles ordered deduplicated context entries from typed
path rules, emits a reason per entry and an explicit retrieval stop condition.

Evidence: 49 CLI tests cover CLI/schema matching, deduplication and reasons.

Remaining: persist manifest checksum and explicit acknowledgements on CL-40 task
records; implement refresh delta semantics in CL-41.
