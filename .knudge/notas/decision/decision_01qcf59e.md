---
id: decision_01qcf59e
type: decision
statement: Provider built-in só llama.cpp (llama-server); demais via GDK do goose; porta HTTP/1.1+2 sem HTTP/3
created_at: 2026-10-13T00:52:06.964Z
body_hash: 84c0c60f
schema_version: 2
tags: [ktu, provider, porta]
anchors: [plan.md]
---
Por quê: simplicidade local e reaproveitar (GDK) em vez de possuir dezenas de dialetos.
Evidência: plan.md §2.3/§2.5; goose-sdk (GDK) carrega o seu próprio transporte; HTTP/3 fica fora.
Consequência: cliente mínimo para o llama-server; HTTP/2 é o teto da porta.