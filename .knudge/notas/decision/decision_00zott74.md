---
id: decision_00zott74
type: decision
statement: "Padrão: simplicidade, FP com DI, logs estruturados, métricas por função e testes rigorosos"
created_at: 2026-10-13T00:52:06.219Z
body_hash: 30ec640b
schema_version: 2
tags: [ktu, metodo]
anchors: [plan.md]
---
Por quê: estabilidade e auditabilidade acima de esperteza; herda a disciplina do knudge (portas + adaptadores, sem unsafe/unwrap).
Evidência: REF/knudge (883 testes, ≤300 linhas/ficheiro) e clippy.toml do repo.
Consequência: domínio puro atrás de portas (Clock/Rng/Fs/Env) com fakes; stdout=dados, stderr=logs; spans zero-cost por função.