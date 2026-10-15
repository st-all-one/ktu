---
id: task_013gz0iw
type: task
statement: "Guard estrutural mínimo: capacidade do AgentSpec, confinamento de path, caps; envelope taint (I13)"
created_at: 2026-10-13T00:51:17.028Z
body_hash: e576ce09
schema_version: 2
tags: [task]
revision: 6
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:16:27.283Z
    notes: coberto por ktu-core (domain/session/agent/guard/budget/testing) + tests/loop.rs; make check verde
  - status: success
    recorded_at: 2026-10-15T09:26:56.940Z
    notes: "ktu-core/src/guard.rs (capacidade do AgentSpec, confinamento de path, caps) + taint.rs (envelope I13 no output de tool); testes tests/loop.rs"
anchors: [crates/ktu-core/src/guard.rs, crates/ktu-core/src/taint.rs]
status: closed
scope: task
---
<!-- knudge:parent task_01xvduai -->