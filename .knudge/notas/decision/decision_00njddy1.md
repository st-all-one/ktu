---
id: decision_00njddy1
type: decision
statement: "Guardrails em 3 camadas: invariantes em código, declarativos em .agents/guardrails, plugins que só acrescentam"
created_at: 2026-10-13T00:52:06.570Z
body_hash: ff2a0767
schema_version: 2
tags: [ktu, guardrails]
anchors: [plan.md]
---
Por quê: uma regra nunca pode ser violada, mesmo que o modelo tente (tese herdada do katu).
Evidência: plan.md §2.8; ordem fixa kernel->gates de plugin->execução; fail-closed.
Consequência: ficheiro de guardrails inválido impede o arranque; plugins nunca transformam Deny em Allow.