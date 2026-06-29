# Install Runtime

This directory contains installer metadata and installer scripts for the
agent-first harness.

Install from a checkout:

```bash
.agent-harness/install/install-harness.sh --directory /path/to/target --yes
```

Windows:

```powershell
.\.agent-harness\install\install-harness.ps1 -Directory C:\path\to\target -Yes
```

Installed runtime paths:

- `AGENTS.md`
- `.agent-harness/`
- `.agent-harness/bin/harness-cli`
- `.agent-harness/scripts/schema/`
- `.gitignore`

Do not install files into `scripts/` or `docs/` in the target repository.
