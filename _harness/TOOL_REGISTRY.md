# Tool Registry

The harness deals with two distinct kinds of "tool". Keep them separate.

|            | Capability manifest (outbound)      | Inbound tool registry                      |
| ---------- | ----------------------------------- | ------------------------------------------ |
| Direction  | harness offers it to the agent      | a project equips it for the harness to use |
| Examples   | the `harness-cli` subcommands below | gitnexus, c3, a linter, a deploy check     |
| Presence   | always compiled in                  | optional; may be absent on any machine     |
| If missing | n/a (it is the harness)             | clean skip; never blocks the main process  |

This document describes both. The **inbound registry** is the extension base: it
is where the harness learns what extra capability is equipped, what purpose it
serves, and whether it is actually present right now, so a workflow step can
adapt to what is installed without the core ever depending on it.

## Inbound Registry: Register A Tool

```bash
_harness/bin/harness-cli tool register \
  --name deploy-check \
  --kind cli \
  --capability deploy-verification \
  --command ./scripts/deploy-check.sh \
  --description "Verify deploy health before release" \
  --responsibility Verification \
  --args "env:enum:required:staging,production"
```

Fields specific to inbound tools:

- `--kind` — how the tool is reached and probed. One of `cli`, `binary`, `mcp`,
  `skill`, `http`. Defaults to `cli`. The kind tells each agent runtime what it
  can orchestrate (a non-Claude agent simply treats a `skill` it cannot run as
  absent) and tells `tool check` which probe to use.
- `--capability` — the workflow purpose a step looks the tool up by. Free-text
  but normalized to kebab-case, so `Impact Analysis`, `impact_analysis`, and
  `impact-analysis` all register as `impact-analysis`. This is the only coupling
  between a step and a tool; steps reference the capability, never the tool
  name.
- `--scan` — for `mcp`/`skill`/`http`, a declarative path or URL that
  `tool check` resolves to decide presence (e.g. `.c3`, `~/.claude/skills/c3`,
  `https://localhost:8080/health`). `cli`/`binary` are probed via their command.

`--force` is only needed for `cli`/`binary` whose command is intentionally
absent on the current machine. `mcp`/`skill`/`http` are not on `PATH` by nature,
so they register without `--force`; their presence is resolved later by
`tool check`.

Registering an MCP server or a Claude skill (examples):

```bash
_harness/bin/harness-cli tool register --name gitnexus --kind mcp \
  --capability impact-analysis --scan ".gitnexus" --command "mcp:gitnexus" \
  --description "Code-graph blast radius" --responsibility Verification
_harness/bin/harness-cli tool register --name c3 --kind skill \
  --capability impact-analysis --scan ".c3" --command "skill:c3" \
  --description "Component model and drift audit (Claude skill)" \
  --responsibility Verification
```

Remove a tool with:

```bash
_harness/bin/harness-cli tool remove --name deploy-check
```

## Inbound Registry: Check Presence

Registration records intent. `tool check` reconciles intent with reality by
scanning each registered tool and persisting the verdict (`status` and
`checked_at`). Run it at intake start so status reflects current reality.

```bash
_harness/bin/harness-cli tool check            # scan all registered tools
_harness/bin/harness-cli tool check --name c3  # scan one
_harness/bin/harness-cli tool check --json     # machine-readable for agents
```

When no external tools are registered, human-readable `tool check` output says
optional tool capabilities are inactive. `--json` still returns an empty array.

Probe per kind:

| Kind            | Probe                                            | `present` means             |
| --------------- | ------------------------------------------------ | --------------------------- |
| `cli`, `binary` | command resolves on `PATH` or as a path          | installed and runnable      |
| `mcp`, `skill`  | `scan_target` path resolves (`~` expands)        | equipped/configured on disk |
| `http`          | `scan_target` reachable over TCP (2s), else path | endpoint answers            |

`tool check` always exits `0`: a missing extension is a fact to report, not a
CLI failure. A `cli`/`binary` is `present` when runnable. An
`mcp`/`skill`/`http` `present` means **equipped** (config/file resolves), not
**live this session** — the agent still confirms live usability at call time,
since only the agent runtime can see whether its MCP server is actually
connected. With no `scan_target`, the status is `unknown` and the agent must
confirm.

## Inbound Registry: Look Up By Capability

A workflow step asks "what is present for this purpose?" rather than naming a
tool:

```bash
_harness/bin/harness-cli query tools --capability impact-analysis
_harness/bin/harness-cli query tools --capability impact-analysis --status present
```

The result is the set of providers. Multiple tools may provide one capability
(gitnexus and c3 both serve `impact-analysis` and are complementary), so a step
reads the set and degrades on how much of it is present.

### Degrade Ladder

The CLI reports facts (`status`); the agent applies policy. The generic rule,
keyed on the present-provider count for a capability:

| Providers present                | Posture  | Agent behavior                                                     |
| -------------------------------- | -------- | ------------------------------------------------------------------ |
| none registered                  | Inactive | clean skip; note `capability X: inactive` in the trace. Not drift. |
| registered but none/some present | Degraded | run with what resolves; set the `Weak proof` flag; note the gap.   |
| all present                      | Full     | normal operation.                                                  |

A registered tool that scans as `missing` is a failed validity gate, not a skip.
A capability with no registered providers is simply inactive and is skipped
without penalty — this is what keeps the core seamless on a fresh install.

### Recommended Capability Vocabulary

Capability is open (no code change to add one), but a step and its providers
must agree on the exact string. Reuse these where they fit before coining a new
one; coin new ones in kebab-case:

```
impact-analysis · deploy-verification · coverage · security-scan
performance-benchmark · documentation-lookup
```

## Inspecting The Registry

```bash
_harness/bin/harness-cli query tools --summary
_harness/bin/harness-cli query tools --json
_harness/bin/harness-cli query tools --responsibility Verification
```

JSON records carry `kind`, `capability`, `scan_target`, `status`, and
`checked_at` alongside the existing fields, so any agent can read the registry
without parsing the human table.

## Compiled Harness Commands (Outbound Manifest)

The executable manifest is authoritative and changes with the CLI build:

```bash
_harness/bin/harness-cli workflow commands --json
```

Use command help for arguments and semantics:

```bash
_harness/bin/harness-cli <command> --help
```

The required lifecycle is `task start`, context acknowledgement, `proof run`,
`task trace`, and `task finish`. Do not invoke retired compatibility commands
such as `init`, `migrate`, standalone `intake`/`trace`, or mutable `story`
subcommands.

## Validation Rules

- Tool names must be unique among registered tools.
- Descriptions must be 10-200 characters.
- Responsibilities must match the Runtime Substrate responsibility list.
- `--kind` must be one of `cli`, `binary`, `mcp`, `skill`, `http`.
- `--capability` must be kebab-case (lowercase letters, digits, single hyphens);
  spaces and underscores are normalized to hyphens.
- `--args` entries must use `name:type:required` or `name:type:required:help`,
  with `required` or `optional` as the third field.
- For `cli`/`binary`, the command must exist as a path or on `PATH`, unless
  `--force` is supplied. `mcp`/`skill`/`http` skip this check.
