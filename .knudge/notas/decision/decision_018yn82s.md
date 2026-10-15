---
id: decision_018yn82s
type: decision
statement: Subagentes read-only (explorer) com fan-out e modo plano read-only estrutural
created_at: 2026-10-13T00:52:07.130Z
body_hash: 7b70d975
schema_version: 2
tags: [ktu, subagentes]
anchors: [plan.md]
---
Por quê: delegar exploração preserva o contexto do agente principal; planejar antes de executar em tarefas longas.
Evidência: plan.md §2.6/§2.13; inspirado no opencode (mode: subagent, deny * + leitura).
Consequência: read-only por conjunto de tools (não prompt); profundidade 1; plan->build exige aprovação.