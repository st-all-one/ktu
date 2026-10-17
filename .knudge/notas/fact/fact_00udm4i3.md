---
id: fact_00udm4i3
type: fact
statement: agent/support.rs dividido: emissao de eventos movida para agent/logging.rs (limite 300 linhas)
created_at: 2026-10-17T01:24:04.480Z
body_hash: f637c0a7
schema_version: 2
anchors: [crates/ktu-core/src/agent/logging.rs]
---
A camada de logging do loop (log_event + log_* por evento) saiu para ktu-core/src/agent/logging.rs; support.rs reexporta o que support/delegation usam e mantem so a logica de turno. Respeita o limite de 300 linhas/ficheiro sem perder coesao.