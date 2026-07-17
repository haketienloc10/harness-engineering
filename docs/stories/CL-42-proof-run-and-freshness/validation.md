# CL-42 Validation

## Result

All CL-42 completion gates are covered. Structured proof rows are append-only,
carry complete execution/output provenance, derive freshness fail-closed, and
drive the story matrix without new direct proof-boolean writes.

## Automated Coverage

`cargo test --workspace` passes 68 tests. The lifecycle fixture runs `git
--version` and an invalid Git subcommand for the same task/story and retains
both rows. It verifies executable/argv, story, HEAD, branch, dirty fingerprint,
CLI/platform, command digest, stdout/stderr and artifact provenance through
`proof query`.

The same fixture proves these freshness transitions:

- matching HEAD, branch, dirty fingerprint, output hashes and artifact hash are
  fresh;
- changing the artifact makes artifact freshness false, while dirty freshness
  also reflects that tracked worktree change;
- a mismatched recorded branch is stale;
- changing retained stderr makes output provenance stale;
- an untracked worktree file makes dirty freshness stale.

The output-limit unit case writes more than 1 MiB, retains exactly 1 MiB and
verifies the stored SHA-256 against retained bytes. Migration coverage verifies
the v11 columns, clean v10-to-v11 upgrade, checksum lineage, migration-framework
rollback and ahead-database refusal.

Matrix coverage proves the latest structured `unit` pass derives `unit=1` and
the later structured `integration` failure derives `integration=0`. A direct
`story update --unit 1` returns the legacy-only error, while imported legacy
numeric columns remain readable when no structured row exists.

## Packaged Black-Box Proof

A temporary `HARNESS_DB` completed `init`, added and linked a normal story/task,
then ran:

```text
proof run --task TASK-000001 --story CL-PACKAGED --layer unit \
  --artifact docs/stories/CL-42-proof-run-and-freshness/validation.md \
  --json -- git --version
```

The packaged binary returned `pass` with HEAD `3d6b68d`, branch
`feature-rework`, 64-character stdout/stderr and artifact hashes, and ignored
repo-relative output paths. `proof query --json` exposed the linked story,
`harness/proof-summary/v2`, dirty fingerprint, CLI/platform and command digest.
`task status --json` reported `head_fresh`, `branch_fresh`, `dirty_fresh`,
`output_fresh` and `artifact_fresh` all `true`. `query matrix --numeric`
reported `unit=1` for `CL-PACKAGED`.

## Verification Commands

```text
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
./install-harness-cli.sh
_harness/bin/harness-cli migrate
_harness/bin/harness-cli doctor --json
```

The packaged smoke additionally validates its JSON fields with `jq` and its
derived matrix row with an exact anchored match. No CL-42 validation gap
remains.
