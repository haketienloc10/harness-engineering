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

## CL-61 Observation Snapshot

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

## Coverage Boundary

Audit names every check it executes and separately reports unknown coverage.
The current unknown set includes deeper Markdown/DB field parity, path-scoped
proof freshness across later commits, generated/installed parity, capsule
fresh-rebuild parity, and latency/over-read/manual-correction telemetry. Zero
findings means no debt in checked coverage only; it never means perfect
maturity.
