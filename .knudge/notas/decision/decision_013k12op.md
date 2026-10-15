---
id: decision_013k12op
type: decision
statement: "Neste host CPU-only, usar o pack PTQ1_0 (5.95 GB) do Bonsai 2 27B com contexto 65536"
created_at: 2026-10-13T01:36:14.745Z
body_hash: d7682fb7
schema_version: 2
superseded_by: decision_009zx5ir
replaces: [decision_0057ju4m]
revision: 3
anchors: [plan.md]
status: superseded
---
Por quê: nao ha GPU dedicada; a memoria e apertada (~7 GiB livres) e o decode CPU e limitado por largura de banda. PTQ1_0 move ~17% menos dados por passo e ocupa menos 1.26 GB que PQ2_0 (7.21 GB), deixando espaco para a KV cache; a model card recomenda PTQ1_0 onde a memoria e mais apertada. Evidência: tabela de memoria/throughput da model card. Consequência: subir contexto (ate 131072) so se medir RAM suficiente; reasoning effort medium para respostas curtas.