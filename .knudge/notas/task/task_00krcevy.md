---
id: task_00krcevy
type: task
statement: Métricas de performance por função (spans zero-cost atrás de feature instrument) com export JSON
created_at: 2026-10-13T00:50:48.140Z
body_hash: 99b321bd
schema_version: 2
tags: [task]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:08:02.552Z
    notes: "porta Metrics (Metric/Metrics/MetricsSnapshot Serialize) + fakes; macro instrument! custo zero desligada por omissao, feature instrument liga-a; teste passa com --features instrument e clippy verde com a feature"
anchors: [crates/ktu-core/src/ports/metrics.rs]
status: closed
scope: task
---
<!-- knudge:parent task_00qm4eu6 -->