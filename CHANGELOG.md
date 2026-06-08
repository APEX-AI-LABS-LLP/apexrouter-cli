# Changelog

## [0.10.0] - 2026-06-08

First public release. Wayland Core is a domain-agnostic autonomous-agent engine written in Rust: terminal-first, multi-provider, MCP-native, and embeddable. It ships as a **public beta**, capable and open, and still hardening under a continuous endurance soak (see "Built to endure" in the README).

### Highlights

* **Multi-provider.** 7 native provider integrations (Anthropic, OpenAI, Google Gemini, Google Vertex AI, AWS Bedrock with SigV4, Cohere, Azure OpenAI) plus a 104-entry models.dev catalog, all behind one provider-neutral engine and a declarative ProviderCompat layer. Circuit-breaker resilience, mid-stream reconnect, and multi-key rotation across every API-key provider.
* **Orchestration.** Sub-agents, a git-worktree-isolated parallel swarm with a dirty-tree guard, declarative ForgeFlows workflows that lower onto the engine's own execution graph, and selectable reducers via `wayland swarm --reduce mesh|fleet|consensus|debate`.
* **Security by default.** A fail-closed OS-native sandbox (bubblewrap, sandbox-exec, AppContainer), a CI-enforced egress chokepoint with an exfil-shape classifier, an always-on SSRF and metadata floor, and argv-safe shell execution.
* **Extensibility.** MCP in both directions (a client, and a server that advertises and executes its own built-in tools, with runtime injection), roughly 70 built-in tools, skills, blocking lifecycle hooks, and a plugin API.
* **Embeddable.** A typed JSON-Lines protocol drives the engine headlessly behind a host app.
* **Self-evolution (GEPA).** A scored optimizer that evolves prompts and skills against your own reference cases.

### Surfaces

One binary, three ways to run it: a one-shot command, an interactive TUI, or a headless JSON stream.

### Notes

This is a public beta. APIs and behavior may change before 1.0. A continuous, fault-injected endurance trial is ongoing; the method, measurements, and honesty bounds are documented in [docs/resilience.md](docs/resilience.md).
