# CL-21 Context Compiler

Status: completed

`workflow context` compiles an ordered, deduplicated, checksummed shadow
manifest from typed lane, phase, path, flag and linked-artifact inputs. It
emits separate must/should/skip lists, a reason per entry, an explicit stop
condition and a lane token-budget hint.

Evidence: 53 CLI tests cover lane/phase goldens, CLI/schema matching,
deduplication, priority promotion, aliases, unrelated-document exclusion and
command-manifest parity. Runtime `score-context` evaluates the same compiled
manifest instead of a separate hard-coded policy table.

Amended ownership: CL-40 persists manifest/checksum on task records; CL-41
implements explicit acknowledgement and refresh delta semantics. These are no
longer completion criteria for this pure compiler story.

Rollback: remove the typed context extensions and restore the prior
path-trigger-only renderer. No task or database state was written by CL-21.
