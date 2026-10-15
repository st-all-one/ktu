---
id: fact_00db9dr6
type: fact
statement: "ktu init cria .agents/{skills,guardrails,memory} e .ktu/{sessions,plugins,trash,plan} de forma idempotente"
created_at: 2026-10-13T01:09:42.512Z
body_hash: c10a6237
schema_version: 2
anchors: [crates/ktu-core/src/layout.rs, crates/ktu-cli/src/init.rs]
---
Evidência: ktu_core::layout (plano puro, golden) + ktu-cli/src/init.rs (efeito na borda); teste init_creates_layout_idempotently. Consequência: o layout de conhecimento vs runtime (§2.11) fica disponível desde F0.