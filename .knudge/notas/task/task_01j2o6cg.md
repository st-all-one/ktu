---
id: task_01j2o6cg
type: task
statement: Test double HTTP com fixtures para exercitar streaming/parsing sem modelo real
created_at: 2026-10-13T00:50:48.716Z
body_hash: f6cb090b
schema_version: 2
tags: [task]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:50:49.554Z
    notes: Test double HTTP em ktu-provider/tests/http_double.rs: servidor TCP minimo que replica bytes de fixture (tests/fixtures/chat_stream.sse) e fecha; exercita reqwest+SSE+acumulador reais. 2 testes: streaming acumula content/reasoning/tool_call; retry 500->200. Verdes.
anchors: [crates/ktu-provider/tests/http_double.rs, crates/ktu-provider/tests/fixtures/chat_stream.sse]
status: closed
scope: task
---
<!-- knudge:parent task_00sv9qlq -->