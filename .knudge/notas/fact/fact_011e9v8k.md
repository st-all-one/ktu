---
id: fact_011e9v8k
type: fact
statement: O ktu compacta contexto de forma aditiva: SessionLog guarda um corte e derive_messages resume os eventos antigos
created_at: 2026-10-13T03:02:28.999Z
body_hash: bcfcd2e3
schema_version: 2
anchors: [crates/ktu-core/src/compact.rs]
---
Evidência: crates/ktu-core/src/compact.rs e session.rs. keep_from_for_turns mantem os ultimos KEEP_TURNS=4; summarize produz um resumo deterministico. O log cru permanece no disco (fonte da verdade); o corte e reproduzivel e reversivel. O loop compacta uma vez quando o orcamento nao cabe (evento session.compact) em vez de falhar.