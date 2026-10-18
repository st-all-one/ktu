---
id: decision_01r1mcjy
type: decision
statement: Gates da Fase 2 (G10-G15): provar usabilidade em cadeias extensas e na mesma sessao
created_at: 2026-10-17T23:01:23.677Z
body_hash: d3194eab
schema_version: 2
anchors: [wiki/implementation_phase2/GATES.md]
---
Decisao: a Fase 2 exige gates proprios (G10-G15), para alem do G4 (quality_test p1-p4). Objetivo: provar a usabilidade em cadeias de interacao MAIS LONGAS e NA MESMA SESSAO, sob as condicoes das 4 utilizacoes. G10 cadeia longa numa mesma sessao (>=15 turnos, artefacto correto, session.verify limpo, sem estagnacao, pin_last_user); G11 robustez multi-modelo (2.o modelo, tool-call em texto, mesma qualidade); G12 independencia do caminho M4 (log byte-identico por run/REPL/crate); G13 pipeline concorrente (N independentes, sem corrupcao, ordenado, lock); G14 project-bound (init idempotente, precedencia, M3 auth-nao-ecoa, M5 guardrail-global-nao-removivel); G15 eixo de memoria (ask --agent, rebuild, projecao read-only). Principios: provar nao afirmar (artefacto), cadeia nao pergunta unica, adversarial, deterministico, bar=equivalencia de grau. Gate de saida da Fase 2: G10-G15 verdes + make check + make supply-chain. Issue H12=task_00h7sxsu, 6 tarefas. Doc: wiki/implementation_phase2/GATES.md.