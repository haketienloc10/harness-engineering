# Harness Maturity Evidence

This source-only report defines maturity as an observed outcome claim. Command,
file or schema existence is capability evidence only.

## Runtime Authority

Run:

```bash
_harness/bin/harness-cli audit --json
```

The runtime report is derived from task closure gates, task-linked traces and
terminal structured-friction outcomes. `doctor` remains the separate authority
for repository and database health.

## H5 Gate

H5 is achieved only when all current runtime conditions pass:

| Evidence | Threshold |
| --- | --- |
| Evidence-backed terminal tasks | 10 |
| Tiny tasks | 3 |
| Normal tasks | 4 |
| High-risk tasks | 2 |
| `blocked-resumed` trace action | 1 |
| `fresh-clone-rebuild` trace action | 1 |
| `installer-upgrade` trace action | 1 |
| Completed normal/high-risk tasks meeting closure gates | 100% |
| Measured improvements | 2 |

A measured improvement must have a terminal status (`validated`, `ineffective`
or `reverted`) and non-empty baseline, predicted metric, observation window and
actual outcome. This keeps ineffective/reverted outcomes visible as learning
without misreporting them as validation success.

Scenario evidence uses exact values in `trace.actions_taken`; summaries and
notes are never heuristically classified.

## Observation History

### CL-61 pre-closure snapshot

The pre-closure `TASK-000011` snapshot reports:

| Evidence | Observed | Required | Result |
| --- | ---: | ---: | --- |
| Evidence-backed terminal tasks | 9 | 10 | gap |
| Tiny tasks | 4 | 3 | met |
| Normal tasks | 2 | 4 | gap |
| High-risk tasks | 3 | 2 | met |
| Blocked/resumed | 0 | 1 | gap |
| Fresh clone/rebuild | 0 | 1 | gap |
| Installer upgrade | 0 | 1 | gap |
| Expanded tasks meeting gates | 5 | 5 | met |
| Measured improvements | 2 | 2 | met |

Current H5 status is therefore `not_achieved`.

### CL-70 terminal snapshot

| Evidence | Observed | Required | Result |
| --- | ---: | ---: | --- |
| Evidence-backed terminal tasks | 13 | 10 | met |
| Tiny tasks | 4 | 3 | met |
| Normal tasks | 4 | 4 | met |
| High-risk tasks | 5 | 2 | met |
| Blocked/resumed | 1 | 1 | met |
| Fresh clone/rebuild | 2 | 1 | met |
| Installer upgrade | 3 | 1 | met |
| Expanded tasks meeting gates | 9 | 9 | met |
| Measured improvements | 2 | 2 | met |

Current H5 status is `achieved`. The outcome follows terminal task gates,
normal-lane qualification tasks and exact trace markers; it is not inferred
from installed commands or test fixtures.

## Coverage Boundary

Audit names every check it executes and separately reports unknown coverage.
Before CL-70 the unknown set included deeper Markdown/DB field parity,
path-scoped proof freshness, generated/installed parity, capsule fresh-rebuild
parity, and latency/over-read/manual-correction telemetry. Audit promotes those
items to checked coverage only after a completed CL-70 task has a passing
`release` proof and all required exact trace markers.

CL-70 terminal audit has no unknown coverage, but still reports unrelated
historical debt: legacy planned stories, abandoned `TASK-000006` without a
trace, and unrooted trace `#2`. H5 and audit-debt status remain separate claims.
