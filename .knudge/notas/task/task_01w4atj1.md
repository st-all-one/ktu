---
id: task_01w4atj1
type: task
statement: "Lock de turno por id (.ktu/sessions/<id>.lock), stale e retry"
created_at: 2026-10-17T22:54:10.120Z
body_hash: b5a80818
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T23:58:04.567Z
    notes: "Lock por id (.ktu/sessions/<id>.lock) com create_new (atomico), stale 30s via porta Clock, recusa se vivo (nunca apaga lock alheio); guarda RAII larga o lock; adquirido so em sessoes persistentes"
anchors: [wiki/backlog/modo-efemero.md]
status: closed
scope: task
---
<!-- knudge:parent task_017rrpwn -->