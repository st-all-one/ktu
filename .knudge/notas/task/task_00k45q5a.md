---
id: task_00k45q5a
type: task
statement: "Teste: dois run no mesmo id => um espera/recusa, log integro (I1/I2)"
created_at: 2026-10-17T22:54:10.452Z
body_hash: 1af20dcf
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T23:58:04.714Z
    notes: "Testes: 3 unitarios do lock (recusa, stale reclamado, ids distintos) + 3 de integracao via binario (lock vivo recusa, lock largado passa e grava log, efemero nao persiste). make check + typos + machete verdes"
anchors: [wiki/backlog/modo-efemero.md]
status: closed
scope: task
---
<!-- knudge:parent task_017rrpwn -->