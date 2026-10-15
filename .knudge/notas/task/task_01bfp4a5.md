---
id: task_01bfp4a5
type: task
statement: derive_messages + projeção de wire (1 passo = 1 assistant com N tool_calls + N tool results)
created_at: 2026-10-13T00:50:48.347Z
body_hash: 3b456950
schema_version: 2
tags: [task]
revision: 6
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:16:27.176Z
    notes: coberto por ktu-core (domain/session/agent/guard/budget/testing) + tests/loop.rs; make check verde
  - status: success
    recorded_at: 2026-10-15T09:26:56.764Z
    notes: derive_messages em ktu-core/src/session/mod.rs + projecao de wire (1 assistant + N tool results) em ktu-provider/src/wire.rs; teste wire_projection_is_deterministic_and_prefix_stable
anchors: [crates/ktu-core/src/session/mod.rs, crates/ktu-provider/src/wire.rs]
status: closed
scope: task
---
<!-- knudge:parent task_01xvduai -->