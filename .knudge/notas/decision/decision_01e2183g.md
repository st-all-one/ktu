---
id: decision_01e2183g
type: decision
statement: Sem optimizacao de micro-caminho: medicao mostra nucleo em microssegundos; gargalo e o modelo
created_at: 2026-10-17T11:49:38.419Z
body_hash: 0bf0a849
schema_version: 2
anchors: [wiki/benchmarks/ktu-bench.json]
---
O benchmark deterministico (make bench) mostra que a projeccao do log (derive_messages) custa 6us/100, 45us/1k e 534us/10k eventos, e um turno de 8 passos 12us. O custo do loop e negligenciavel face a latencia do modelo (inferencia local ou rede). Pela prioridade do plan.md 2 (performance terciaria), nao se optimiza micro-caminho sem numero que o exija; revisitar so se uma medicao real cruzar um limiar (ex.: projecao > 5 ms).