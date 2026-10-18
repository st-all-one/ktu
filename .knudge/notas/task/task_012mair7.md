---
id: task_012mair7
type: task
statement: Contrato de projecao (doc) + projecao do log de sessao em notas derivadas
created_at: 2026-10-17T22:54:32.503Z
body_hash: df2ffa65
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T01:21:21.956Z
    notes: "Contrato de projecao documentado no knudge (wiki/specs/fontes.md, secao \"O contrato de projecao\") e no ktu (wiki/projecao-sessao.md). Projecao implementada do lado do AGENTE (o knudge nao conhece o schema Event): ktu-memory::project (Event -> MemoryDraft, uma nota meta por turno, statement \"[sessao] pedido\", body = resposta + ferramentas, anchors dos args das tools, tags session-log) + ktu memory project (idempotente, temp->fsync->rename). Testes unit + integracao."
anchors: [wiki/knudge-eixo-memoria.md]
status: closed
scope: task
---
<!-- knudge:parent task_00104ugh -->