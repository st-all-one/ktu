---
id: task_00z3gemm
type: task
statement: ktu memory drain (status/digest/force) com fila derivada
created_at: 2026-10-17T22:54:40.004Z
body_hash: 085a52eb
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T01:49:37.185Z
    notes: "ktu memory drain [--status|--digest|--force] (H8). Indice derivado em .agents/memory/.idx/embeddings.jsonl com o MESMO codec do knudge (knudge_core::embeddings::EmbeddingIndex) -> invalida por identidade (provider/modelo/revisao/dimensao/prefixo), auto-repara indice ilegivel (tratado como ausente), escrita temp->fsync->rename. Fila derivada por body_hash: indexed/pending/stale; --digest em lotes de embeddings.batch, idempotente, grava uma vez por lote, falha do provedor NAO descarta nota (exit 1 + pendentes); --force ignora o indice e re-embeda tudo. Config em .agents/memory.toml (schema fechado, chaves do knudge; invalido = exit 1). Testes: ktu-memory/src/drain/tests.rs + ktu-cli/tests/embeddings.rs (servidor HTTP falso, nunca llama-server real)."
anchors: [wiki/backlog/memory-drain-service.md]
status: closed
scope: task
---
<!-- knudge:parent task_01t8rm0l -->