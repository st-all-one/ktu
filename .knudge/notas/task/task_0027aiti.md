---
id: task_0027aiti
type: task
statement: Porta Embedder + FakeEmbedder deterministico + canal semantico no recall
created_at: 2026-10-17T22:54:39.649Z
body_hash: 2c4b6a06
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T01:49:37.008Z
    notes: "Porta Embedder SINCRONA em ktu-core::ports (meta + embed(lote), ordem preservada, EmbedError Unavailable/Invalid) porque o recall do kernel (Memory) e sincrono. Fakes: FakeEmbedder (hash FNV-1a deterministico, normalizado, dim configuravel) e UnavailableEmbedder (prova a degradacao). Adaptador real: ktu-provider::embed::HttpEmbedder (reqwest::blocking, POST /v1/embeddings OpenAI-compatible, ordena por index, valida a dimensao como contrato). Canal vetorial na fusao RRF: JsonlMemory::with_embeddings liga RecallQuery.vector (top_k 20, min_score 0.0 como o knudge); sem provedor/indice desliga SEM AVISO. Prova: ktu-memory/tests/memory.rs::semantic_channel_recovers_a_paraphrase_that_lexical_misses (parafrase sem sobreposicao lexical so e encontrada depois do dreno; warm == again)."
anchors: [wiki/backlog/knudge-embeddings.md]
status: closed
scope: task
---
<!-- knudge:parent task_01t8rm0l -->