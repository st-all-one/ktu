---
id: task_012p12rz
type: task
statement: "Guard estrutural mínimo: capacidade do AgentSpec, confinamento de path, caps; envelope taint no output de tool (I13)"
created_at: 2026-10-13T00:50:48.436Z
body_hash: 78c0b8d8
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:18:44.061Z
    notes: "duplicado do task_013gz0iw (mesma entrega): guard estrutural + confinamento de path + envelope taint, coberto por ktu-core/src/guard.rs, taint.rs e tests/loop.rs"
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_01xvduai -->