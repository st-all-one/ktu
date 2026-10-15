---
id: decision_0166fqyj
type: decision
statement: "Memória = JSONL append-only em .agents/memory, adoptando o contrato do knudge-core"
created_at: 2026-10-13T00:52:06.378Z
body_hash: ab138ac3
schema_version: 2
tags: [ktu, memoria]
anchors: [plan.md]
---
Por quê: REF/knudge é referência bem-sucedida de estruturação de memória; o ktu não reinventa.
Evidência: plan.md §2.7; knudge: id endereçado por conteúdo, índice derivado, write idempotente (0.75/0.92), busca RRF determinística.
Consequência: notes.jsonl é a fonte da verdade; index/ é derivado e apagável; hints de recall são ponteiros, nunca o corpo.