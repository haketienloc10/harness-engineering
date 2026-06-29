# Install Runtime

This directory contains installer metadata for the agent-first harness.

Install into the current repository:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/install.sh?$(date +%s)" | bash
```

Install into another directory:

```bash
curl -fsSL "https://raw.githubusercontent.com/haketienloc10/harness-engineering/main/install.sh?$(date +%s)" | bash -s -- /path/to/target
```

Installed runtime paths:

- `AGENTS.md`
- `.agent-harness/`
- `.agent-harness/bin/harness-cli`
- `.agent-harness/scripts/schema/`
- `.gitignore`

Do not install files into `scripts/` or `docs/` in the target repository.

The installer first tries to install a compatible CLI release binary. If that
binary is unavailable or cannot run on the target system, it falls back to
building from `crates/harness-cli` when the Rust source and `cargo` are
available.
