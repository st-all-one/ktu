---
id: risk_01kwj0u9
type: risk
statement: O pack PQ2_0 exige o fork PrismML-Eng/llama.cpp; o llama.cpp stock produz lixo em silencio
created_at: 2026-10-13T01:34:42.371Z
body_hash: 246f019f
schema_version: 2
revision: 2
anchors: [plan.md]
status: forgotten
---
Por quê: os kernels ternary hybrid-attention (rotação de Hadamard nas activações) só existem no fork. Evidência: a model card diz que o stock rejeita PQ2_0/PTQ1_0 e carrega Q2_0 sem aviso, gerando garbage. Consequência: ktu providers doctor tem de validar o binario/fork e o ficheiro antes de servir; o plano F2 assume o llama-server do fork, nao o stock.