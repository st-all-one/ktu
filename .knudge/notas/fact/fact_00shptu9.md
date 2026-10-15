---
id: fact_00shptu9
type: fact
statement: "Ornith-1.5-9B-GGUF (ornith-ai) e dense ~9B qwen35, quants Q4_K_M 5.78GB a Q8_0 9.79GB, corre em llama.cpp stock"
created_at: 2026-10-13T02:13:48.990Z
body_hash: "967e2640"
schema_version: 2
anchors: [plan.md]
---
Evidência: model card HF. Reasoning: o turno abre com bloco de pensamento exposto em reasoning_content; tool-calls nativos parseados para tool_calls (OpenAI). Contexto ate 262144. Sampling geral: temp=1.0, top_p=0.95, top_k=20, min_p=0.0, presence_penalty=1.5, repetition_penalty=1.0; codigo: temp=0.6, presence_penalty=0.0.