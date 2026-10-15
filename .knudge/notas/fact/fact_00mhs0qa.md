---
id: fact_00mhs0qa
type: fact
statement: "Metricas por funcao usam a macro instrument! (feature instrument), custo zero por omissao"
created_at: 2026-10-13T01:08:07.300Z
body_hash: 483ec25e
schema_version: 2
anchors: [crates/ktu-core/src/lib.rs, crates/ktu-core/src/ports/metrics.rs]
---
Evidência: teste instrument_records_duration_when_enabled passa com cargo test -p ktu-core --features instrument; sem a feature a macro expande so para o corpo. Consequência: instrumentação é opt-in e não pesa no agente normal.