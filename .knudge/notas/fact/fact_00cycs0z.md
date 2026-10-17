---
id: fact_00cycs0z
type: fact
statement: I11: guardrail estrutural primeiro; gates de plugin so se Allow; gate em erro/timeout = Deny
created_at: 2026-10-17T07:02:33.284Z
body_hash: f56e68f4
schema_version: 2
anchors: [crates/ktu-core/src/agent/delegation.rs]
---
agent/delegation.rs::evaluate_guard avalia o StructuralGuard e so consulta os gates de plugin se Allow; um plugin nunca transforma Deny em Allow. PluginHost devolve Deny{plugin_timeout} em erro/timeout (fail-closed). Provado por plugin_gate_denies_and_the_denial_is_evidence e a_plugin_cannot_free_a_structural_deny.