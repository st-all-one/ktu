---
id: fact_017yzhrw
type: fact
statement: "Backlog do knudge completo: 10 paginas (114-123) com dependencia, graph, lifecycle, embeddings, config e git-sync"
created_at: 2026-10-17T13:30:33.310Z
body_hash: 4244031c
schema_version: 2
anchors: [wiki/backlog/README.md]
---
Complemento da revisao do knudge (wiki/revisao-knudge.md): o backlog passou a ter 10 paginas do knudge. Incorporar: 114 Store+notas/eventos+write, 115 handoff::rewind, 116 health::doctor. Avaliar: 117 hierarquia epic>issue>task, 119 grafo avancado (ontologia/TMS/PageRank-PPR/Louvain), 120 ciclo de vida (confianca Beta, shelf-life FSRS, decay, demolicao), 121 porta Embedder (canal semantico). Decidir: 118 como consumir o knudge-core (path local vs git pinada por tag vs submodulo vs publicado; knudge-core e publicavel por D214; recomendado git pinada enquanto nao ha publicacao). Consolidacoes: 119 -> 2 auditoria-densa, 120 -> 6 contexto-rate-distortion, 121 -> 24 contexto-por-modelo, 122 config dois niveis -> 76 config-precedencia, 123 git::sync -> 16 git-versionado. Nao adoptar: binario kd/knudge-mcp como deps de runtime, CLI/envelope, drain service. Backlog total: README + 123 paginas.