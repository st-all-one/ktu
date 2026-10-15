---
id: task_00mabsbe
type: task
statement: Provider falso com fixtures: replay determinístico byte-a-byte (I6)
created_at: 2026-10-13T00:50:48.533Z
body_hash: ca3a65a3
schema_version: 2
tags: [task]
revision: 6
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:16:27.379Z
    notes: coberto por ktu-core (domain/session/agent/guard/budget/testing) + tests/loop.rs; make check verde
  - status: success
    recorded_at: 2026-10-15T09:26:57.127Z
    notes: "FakeProvider/FakeTools roteirizados em ktu-core/src/testing.rs; replay byte-a-byte de fixtures no test-double HTTP (ktu-provider/tests/http_double.rs + fixtures/chat_stream.sse, entregue em G2 task_01j2o6cg)"
anchors: [crates/ktu-core/src/testing.rs, crates/ktu-provider/tests/http_double.rs]
status: closed
scope: task
---
<!-- knudge:parent task_01xvduai -->