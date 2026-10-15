---
id: decision_0057ju4m
type: decision
statement: Modelo de referencia do ktu para llama.cpp: prism-ml/Ternary-Bonsai-2-27B-gguf PQ2_0 em thinking mode
created_at: 2026-10-13T01:34:42.166Z
body_hash: be185edd
schema_version: 2
superseded_by: decision_013k12op
revision: 2
anchors: [plan.md]
status: superseded
---
Por quê: 27B-class com tool calling (BFCL v3 74.92) e 262K de contexto a 7.21 GB, corre num portatil. Evidência: https://huggingface.co/prism-ml/Ternary-Bonsai-2-27B-gguf. Sampling thinking mode: temperature=1.0, top_p=0.95, top_k=20, min_p=0.05, presence_penalty=0.0, repetition_penalty=1.0 (reasoning effort default xhigh; medium para respostas curtas). Consequência: o provider expõe sampling configurável e trata o reasoning (thinking) sem o misturar com a resposta final.