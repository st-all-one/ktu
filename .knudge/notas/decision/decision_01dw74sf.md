---
id: decision_01dw74sf
type: decision
statement: O core puro expoe Logger e Metrics como portas; tracing so existe na borda (ktu-cli)
created_at: 2026-10-13T01:08:07.195Z
body_hash: 87f4b151
schema_version: 2
anchors: [crates/ktu-core/src/ports/logger.rs, crates/ktu-cli/src/logging.rs]
---
Por quê: mantém ktu-core sem I/O (§2.1) e testável com fakes; segue o padrão do knudge-core. Evidência: ktu-core/src/ports/logger.rs e metrics.rs; adaptador em ktu-cli/src/logging.rs. Consequência: nenhum crate puro importa tracing.