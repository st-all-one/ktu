---
id: decision_00h4zpsb
type: decision
statement: BudgetGate é o dono único do teto de contexto; pin da instrução corrente; raw_min alto
created_at: 2026-10-13T00:52:07.284Z
body_hash: 2a0e94de
schema_version: 2
tags: [ktu, contexto]
anchors: [plan.md]
---
Por quê: contexto pequeno causa thrashing e loop (lição do katu).
Evidência: STAGING.md do katu; a tarefa só fecha com ~39-48k tokens de input.
Consequência: atingir o teto é recusa/compactação, nunca cortar evidência; um só dono do teto.