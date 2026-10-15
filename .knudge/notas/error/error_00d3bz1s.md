---
id: error_00d3bz1s
type: error
statement: katu: contexto raw_min=4096 tokens causou thrashing e termination=loop
created_at: 2026-10-13T00:52:07.430Z
body_hash: 7cd29ce4
schema_version: 2
tags: [ktu, error, contexto]
supports: [decision_00h4zpsb]
revision: 2
anchors: [plan.md]
---
Por quê: o prefixo descartava investigação e o modelo relia os mesmos ficheiros.
Evidência: REF/katu STAGING.md (termination=loop no passo 52, 145 calls).
Consequência: subir o teto e fixar a instrução (BudgetGate + pin).