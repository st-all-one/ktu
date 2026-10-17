---
id: fact_01qnm8ob
type: fact
statement: Revisao katu vs ktu: ktu reconstruiu o slice vertical com ~1/5 do codigo e gate verde; katu falhou pela ordem
created_at: 2026-10-17T12:23:35.064Z
body_hash: 7f09acf5
schema_version: 2
anchors: [wiki/revisao-katu-vs-ktu.md]
---
Revisao completa em wiki/revisao-katu-vs-ktu.md. katu (1a tentativa): ~72 800 linhas src, ~1001 testes, 276 pacotes, limite 400 linhas, MSRV 1.97, e ZERO sessoes a funcionar (thrashing de contexto, wire HTTP 400, ordem de construcao invertida: policy/audit/rate-distortion antes do loop). ktu (2a): ~14 200 linhas, 161 testes, 345 pacotes, limite 300, MSRV 1.99, binario 6,0 MB, gate G4 verde com o modelo de referencia. Adoptou do katu: BudgetGate/pin_unit, wire_messages, prompt/skills, taint, stall/StopReason, memory_gate, budget/caps, trash, plan. Deferiu com razao: audit/bloom, confidence/FDR, rate-distortion, diag 122 eventos, MCP, GDK (decidido nao), WASM, sandbox de SO. Acrescentou: guardrails 3 camadas fail-closed, plugins com hot reload por geracao, porta unica + front-ends, subagentes read-only + modo plano, gate de qualidade executavel, packaging/releases, benchmarks com evidencia.