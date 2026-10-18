---
id: task_00t9pviy
type: task
statement: Modelo embeddinggemma-2 Q8_0 (porta 8889) + degradacao graciosa + remove-model para trash
created_at: 2026-10-17T22:54:40.742Z
body_hash: 2c0c5832
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T01:49:37.488Z
    notes: "Modelo google/embeddinggemma-2 (embeddinggemma-2-Q8_0.gguf) na porta 8889, --every 1h; GGUF com revisao PINADA e SHA-256 verificado (mismatch aborta sem instalar nada). Degradacao graciosa em tres niveis: sem memory.toml/provider=none -> recall lexical e --digest nao escreve nada; provedor fora -> fila fica pendente e o comando falha alto; indice ausente/ilegivel -> lexical. --remove-model -> trash recuperavel. DESVIO do plano (documentado): o llama.cpp NAO e descarregado (e pre-requisito do provider e o installer oficial nao e verificavel aqui) - o worker localiza-o no PATH/KTU_LLAMA e imprime o guia; --no-deps salta a descarga do modelo."
anchors: [wiki/backlog/memory-drain-service.md]
status: closed
scope: task
---
<!-- knudge:parent task_01t8rm0l -->