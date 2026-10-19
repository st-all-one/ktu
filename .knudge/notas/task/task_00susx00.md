---
id: task_00susx00
type: task
statement: check-knudge na CI: workspace proprio do knudge verificado (L1)
created_at: 2026-10-18T13:32:19.095Z
body_hash: a3979c1d
schema_version: 2
tags: [fase3]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T15:31:28.046Z
    notes: "make check-knudge (cd crates/knudge && make check) + job knudge na CI com rust-cache sobre crates/knudge; o G22 corre a suite (22 binarios, 0 falhas) e verifica o passo no workflow. Medido: ~6 s quente, ~45 s com alvo vazio."
anchors: [wiki/implementation_phase3/F7-travoes.md]
status: closed
scope: task
---
<!-- knudge:parent task_01nopvll -->