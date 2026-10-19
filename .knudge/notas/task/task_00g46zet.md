---
id: task_00g46zet
type: task
statement: Piso visivel: ktu guardrails list mostra global+local com origem (L6)
created_at: 2026-10-18T13:32:18.139Z
body_hash: bdb06d1e
schema_version: 2
tags: [fase3]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T14:48:33.092Z
    notes: "Piso visivel (L6): ktu guardrails list mostra as regras dos DOIS niveis com origem (global primeiro) e --json traz {id, origin} para todas; o mesmo para commands/plugins e o novo skills ls (M8). Bug evitado: a logica de composicao dos dois niveis estava duplicada (builder + evaluate_fact) - agora e uma so (facade::declarative_rules)."
anchors: [wiki/implementation_phase3/F3-recursos-globais.md]
status: closed
scope: task
---
<!-- knudge:parent task_004bvdzb -->