---
id: decision_009zx5ir
type: decision
statement: O modelo de referencia do ktu passa a ser Ornith-1.5-9B-GGUF Q4_K_M sobre llama.cpp stock
created_at: 2026-10-13T02:13:43.785Z
body_hash: a28f386f
schema_version: 2
replaces: [decision_013k12op, decision_01yhrzu0]
revision: 3
anchors: [plan.md]
---
Por quê: o pack ternary exigia o fork PrismML; o Ornith-1.5-9B (arquitectura qwen35, dense ~9B) corre em llama.cpp stock e traz tool-calling nativo + reasoning, sem complexidade. Escolha do quant para este host CPU-only (17 GiB, ~7 livres, sem GPU): Q4_K_M (~5,78 GB) equilibra memoria/contexto, deixando RAM para a KV cache (Q5_K_M 6,64 e Q6_K 7,56 cabem mais justos). Contexto 32768 com -fa on e KV q8_0; sampling temp=1.0 top_p=0.95 top_k=20 min_p=0.0 presence_penalty=1.5 repetition_penalty=1.0. Evidência: model card Ornith-1.5-9B (SWE-bench Verified 70.6, MCP-Atlas 54.2).