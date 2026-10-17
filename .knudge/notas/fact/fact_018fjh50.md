---
id: fact_018fjh50
type: fact
statement: "ktu-plugin: processo externo JSON-lines sobre stdio, plugin.toml, hot reload por geracao"
created_at: 2026-10-17T07:02:33.821Z
body_hash: 6ff502f5
schema_version: 2
anchors: [crates/ktu-plugin/src/host.rs]
---
crates/ktu-plugin: manifest.rs (plugin.toml: entrypoint/events/mode/timeout_ms/daemon), process.rs (ProcessPlugin spawn+timeout, fail-closed em gate), host.rs (PluginHost: scan, reload incrementa geracao, dispatch fixa o Arc da geracao - I12). Firewall: nao depende de tools/provider/port/memory/cli.