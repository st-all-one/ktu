---
id: task_00ddzsgt
type: task
statement: "Pendencias de organizacao: remover tokio-util morto, resolver task forgotten e cobrir o Logger"
created_at: 2026-10-17T00:42:25.502Z
body_hash: 26c6e23b
schema_version: 2
tags: [cleanup]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T01:06:55.432Z
    notes: "Removidas deps nao usadas (indexmap, tokio-util, thiserror/tracing/serde/serde_json por crate) — machete zero. risk_01cpastn (Logger) confirmado resolvido: guard.decision/tool.result ja emitem pelo Logger."
anchors: [Cargo.toml]
status: closed
scope: task
---
Remover a dependencia tokio-util do workspace (nenhum crate a usa). Fechar/esquecer task_01kf735v (duplicado de task_00o74kgy). Ligar guard.decision e tool.call/tool.result a porta Logger (risk_01cpastn) mantendo stdout=dados e stderr=logs.
<!-- knudge:parent task_015epqsj -->