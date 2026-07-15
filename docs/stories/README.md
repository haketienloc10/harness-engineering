# Stories

Stories are work packets. They turn product intent into bounded implementation
and validation work.

Current story packets may live directly in this directory or under scoped
`epics/` folders as the project grows.

## Normal Story

Use `_harness/templates/story.md` for normal feature work.

Suggested path:

```text
docs/stories/epics/E01-domain-name/US-001-short-story-title.md
```

## High-Risk Story

Use the progressive `_harness/templates/story.md` when the feature intake
classifies work as high-risk.

Suggested path:

```text
docs/stories/epics/E02-risky-domain/US-012-risky-story-title/
  execplan.md
  overview.md
  design.md
  validation.md
```

## Status Flow

New command-first stories use the canonical template vocabulary:

```text
planned -> in_progress -> completed
              |  ^
              v  |
            blocked
```

Existing legacy stories with `implemented`, `changed` or `retired` remain
read-compatible during the compatibility window, but new or updated stories
must use `planned`, `in_progress`, `completed` or `blocked` from
`_harness/templates/story.md`.
