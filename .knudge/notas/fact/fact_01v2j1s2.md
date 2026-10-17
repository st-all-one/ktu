---
id: fact_01v2j1s2
type: fact
statement: Hooks tipados (HookEvent/HookPayload) + Dispatcher determinístico por nome; pontos no loop
created_at: 2026-10-17T07:02:33.007Z
body_hash: "2e325887"
schema_version: 2
anchors: [crates/ktu-core/src/plugin.rs]
---
ktu-core/src/plugin.rs (HookEvent, HookPayload, PluginVerdict, Plugin) + plugin/dispatch.rs (Hooks, HookVerdict, NoHooks, Dispatcher). Pontos: session.start/end, user.input, model.request/output, action.before/after, guard.decision, kernel.error. Cada dispatch entra no log da sessao (Event::PluginDispatch) e na porta Logger (plugin.dispatch).