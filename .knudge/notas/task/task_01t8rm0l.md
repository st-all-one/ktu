---
id: task_01t8rm0l
type: task
statement: H8 — Embeddings locais opcionais (drain service --install)
created_at: 2026-10-17T22:53:39.782Z
body_hash: 92699b57
schema_version: 2
tags: [fase2]
results_in: [task_0027aiti, task_00z3gemm, task_018q1vht, task_00t9pviy]
revision: 7
outcomes:
  - status: success
    recorded_at: 2026-10-18T01:49:37.650Z
    notes: "H8 (embeddings locais opcionais) concluida: porta Embedder + fakes, indice derivado com o codec do knudge, filtros/canal RRF, ktu memory drain (status/digest/force), worker drain service (systemd/launchd + servidor persistente) e memory.toml com as chaves do knudge (uma so verdade). Contrato em wiki/memoria-semantica.md; prova em wiki/implementation_phase2/H8-embeddings.md. Gates: make check + supply-chain + typos verdes no ktu e no knudge. Adiado (YAGNI): cache vetorial, min_score configuravel, modo lazy, similarity=dot."
anchors: [wiki/backlog/memory-drain-service.md]
status: closed
scope: issue
---
<!-- knudge:parent epic_01j7ou2p -->