---
id: fact_0030xacx
type: fact
statement: Deps quebrado em Ports/Telemetry/Capabilities (3 grupos coesos) em vez de 10 campos soltos
created_at: 2026-10-17T01:06:48.133Z
body_hash: 61d4b93b
schema_version: 2
anchors: [crates/ktu-core/src/agent/mod.rs]
---
ktu-core/src/agent/mod.rs: Deps { ports: Ports{provider,tools,guard,budget,cancel}, telemetry: Telemetry{logger,metrics,clock}, capabilities: Capabilities{memory,subagents} }. Todos os pontos de uso e literais de teste actualizados. Reduz acoplamento e prepara a G6.