# Agent Instructions

## Authority

Apply sources in this order:

1. Current user instruction
2. `docs/product/`
3. `docs/stories/`
4. `_harness/bin/harness-cli query matrix`
5. `docs/decisions/` and durable CLI decisions
6. Code, tests, then historical material

Treat `_harness/` as agent runtime tooling. Do not let it override product
truth or current user intent.

## Execute

1. Before any edit, run `_harness/bin/harness-cli task start` with the correct
   `--type`, flags, owner, and session.
2. Complete the returned context, story, decision, and approval gates before
   implementation. Read only the references selected by that context.
3. Make the smallest scoped change. Keep the linked product/story/decision
   record current.
4. Run validation, record it with `proof run`, record a task trace, render a
   capsule when required, then close only with `task finish`.
5. Use command-first lifecycle state only. Never create or edit the operational
   database directly.

For command semantics, lanes, context rules, trace fields, and proof policy,
read the owning `_harness/` reference instead of adding explanation here.

## Continue

For vague continuation prompts, run:

```bash
_harness/bin/harness-cli task next --json
```

- Do not implicitly resume a task, acquire a lease, or start a backlog item.
- If the task belongs to another owner/session, require explicit handoff
  authority.
- If only a backlog item is returned, present it and obtain confirmation before
  starting it.

## Stop

Request human approval before high-risk direction, credentials, cost, destructive
actions, or any workflow-required gate. If the CLI is missing, use Markdown
artifacts, run available validation, and record `harness-cli` missing as
friction; do not construct the database.

## Respond

- Reply in Vietnamese unless the user explicitly requests another language.
- Preserve technical terms, code, commands, paths, config keys, and errors.
- Lead with the outcome. Be concise and operational.
- For decisions, risks, or failures, state: event → cause → impact → recommended
  action. Use a concrete example when it removes ambiguity.
