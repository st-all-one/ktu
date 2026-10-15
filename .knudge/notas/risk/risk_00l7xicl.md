---
id: risk_00l7xicl
type: risk
statement: Modelo local fraco em tool-calling (granite-97m é pequeno demais)
created_at: 2026-10-13T00:52:07.881Z
body_hash: 3ffaf2a5
schema_version: 2
tags: [ktu, risk, provider]
anchors: [plan.md]
---
Por quê: a qualidade do resultado depende do modelo local.
Evidência: plan.md §5.
Consequência: exigir GGUF com tool-calling (4-8B); testes do loop usam provider falso.