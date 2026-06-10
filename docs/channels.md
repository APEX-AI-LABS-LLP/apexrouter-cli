# Channels — inbound security model

wayland-core can receive messages from chat platforms (Telegram, Discord,
Slack, Signal, …) and answer them with an agent turn. Because a channel
sender is **remote** — and, depending on your access policy, possibly
untrusted — inbound traffic passes through two independent security gates
before and around the agent turn:

1. **Access policy** — *who* may drive the agent (fail-closed allowlists).
2. **Tool posture** — *what the agent may touch* on the host (no filesystem
   or shell by default).

Both are configured per channel in that channel's config file under
`~/.wayland/channels/<name>.toml`, in the `[inbound]` table.

> If `[inbound]` is absent, the channel is **fail-closed**: every inbound
> message is denied. Inbound dispatch does nothing until you opt in.

---

## Access policy — who may drive the agent

```toml
# ~/.wayland/channels/tg.toml
platform = "telegram"

[inbound]
dm = "allowlist"                 # open | allowlist | pairing | disabled
dm_allowlist = ["123456789"]     # stable platform sender ids; "*" = anyone
group = "disabled"               # open | allowlist | disabled
require_mention = true           # in groups, only act when addressed
```

Defaults (used for any unset field) are the fail-closed posture:
`dm = "allowlist"` with an **empty** `dm_allowlist` (so no one is
permitted), `group = "disabled"`, `require_mention = true`.

**Lock `dm_allowlist` to specific sender ids.** `dm_allowlist = ["*"]`
opens DMs to *anyone who can find the bot* — only use it for a throwaway
test bot, never a deployment. To allow a specific person, add their stable
platform `sender_id` (e.g. their Telegram numeric user id):

```toml
dm = "allowlist"
dm_allowlist = ["123456789"]     # only this user may DM the bot
```

Allowlist semantics: a list permits an id **iff** it contains the literal
`"*"` (wildcard) **or** the exact id. An empty list permits nothing. Group
acceptance under `group = "allowlist"` requires BOTH the group
(`group_allowlist`) AND the sender (`sender_allowlist`) to be listed.

---

## Tool posture — what the agent may touch

A channel turn runs a real agent engine on your host. Without scoping, the
built-in `Read`/`Grep`/`Glob` tools (which are auto-approved) would let a
remote sender read host secrets and have the reply ship them back. The
`tools` posture controls which tools a channel-originated engine is built
with:

```toml
[inbound]
tools = "conversational"         # conversational (default) | workspace | full
tool_workspace_root = "/srv/agent-workspace"   # only used by "workspace"
```

| Posture | Filesystem / shell | Use when |
|---|---|---|
| **`conversational`** (default) | **None.** Only conversational/network tools (and operator-wired MCP servers) are exposed. | A chat bot that answers questions, calls APIs, and uses your MCP tools — but must never touch the host filesystem. |
| **`workspace`** | `Read`/`Write`/`Edit`/`Grep`/`Glob` are available but **jailed** to `tool_workspace_root` (a remote sender cannot read or write outside it). Shell/exec tools (`Bash`, `Git`, `kubectl`, …) stay **unavailable** — they bypass the jail. | A confined "do real work in this directory" agent reachable over chat. |
| **`full`** | **Everything**, host-wide — identical to a local CLI session. | Trusted, locked-down deployments only. Dangerous for any publicly-reachable channel. |

Notes:

- The posture is enforced at the tool registry, so a dropped tool is
  **un-dispatchable** — not merely hidden from the model. Even a
  hallucinated call cannot reach it.
- `tool_workspace_root` defaults to the agent's working directory when
  unset under `workspace`.
- The posture applies **only** to channel-originated engines. Your local
  CLI / TUI / `--json-stream` sessions always keep the full toolset.
- **MCP caveat:** operator-wired MCP servers are kept under
  `conversational` and `workspace` (they are deliberate, named
  extensions). If an MCP server itself exposes host filesystem access,
  threat-model that channel as `full`-equivalent.

---

## Recommended deployment baseline

```toml
[inbound]
dm = "allowlist"
dm_allowlist = ["<your-platform-user-id>"]
group = "disabled"
require_mention = true
tools = "conversational"
```

This admits only you, in DMs, with no host filesystem or shell exposure.
Widen deliberately from there.
