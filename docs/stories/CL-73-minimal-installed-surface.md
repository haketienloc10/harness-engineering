# CL-73 Minimal Installed Harness Surface

## Status

completed

## Lane and Scope

- Lane: high-risk
- Risk flags: architecture-direction, source-hierarchy, installer-state
- In scope: replace directory-wide installer copying with an explicit minimal
  Harness payload for target repositories.
- Out of scope: deleting this repository's historical records or changing CLI
  compatibility behavior.

## Product Contract

Installing Harness into another repository adds only the command-first runtime:
the executable CLI, workflow configuration and command manifest, templates, and
schema migrations. It embeds the Harness block in `AGENTS.md` but does not copy
source-repository documentation, stories, decisions, proposals, editor config,
or target-product records.

## Acceptance Criteria

- The installer uses an allowlist rather than copying `_harness/` and `docs/`
  broadly then excluding files.
- A target install contains only the declared Harness runtime payload plus its
  generated local integration files.
- Existing target files and `harness.db` remain unchanged.
- Installer state-safety verification passes.

## Design and Decisions

The user requested an installer that copies only valuable files. For this
change, "valuable" means files needed to execute, configure, validate, extend,
or initialize the command-first Harness runtime. Historical Markdown and
source-only policy/reference documents remain in the source repository but are
not installed.

## Human Gates

- `architecture-direction` and `source-hierarchy`: user request on 2026-07-16,
  "viết lại install.sh ... chỉ thực hiện cài đặt các file có giá trị".

## Validation and Evidence

| Layer | Expected proof | Result |
| --- | --- | --- |
| Integration | `bash tests/installer_state_safety.sh` | pass |
| CLI | installed `workflow validate` and parity checks | pass |
| Static | `bash -n install.sh` and `git diff --check` | pass |

## Rollback and Harness Delta

Revert the installer and its safety test together. The target installer remains
non-destructive outside its owned `_harness/` runtime payload and generated
integration block.

2026-07-17 follow-up: the generated target block now uses Vietnamese,
agents-first installation instructions while preserving byte-for-byte parity of
the canonical shared `AGENTS.md` segment. Installer state-safety proof asserts
both the parity segment and the generated Vietnamese runtime contract.
