---
id: task_016trqis
type: task
statement: T2: thread de tools dedicada (mpsc + oneshot); kernel nunca corre tools na sua task
created_at: 2026-10-17T00:42:23.966Z
body_hash: 0fb77959
schema_version: 2
tags: [t2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T01:06:55.071Z
    notes: "ToolThread (ktu-tools/src/tool_thread.rs): T2 nomeada, executor construido dentro da thread, mpsc+oneshot, handshake de specs, Drop faz join. CLI usa-a. Testes tools_run_on_a_dedicated_thread (ThreadId distinto) e tool_errors_are_contained verdes; make check verde."
anchors: [crates/ktu-tools/src/tool_thread.rs]
status: closed
scope: task
---
Implementar ToolThread em ktu-tools: thread nomeada ktu-tools-t2 que constroi o Toolbox e serve pedidos por tokio::sync::mpsc (unbounded) + tokio::sync::oneshot para a resposta. O kernel (T1) fala apenas por mensagens (I8). Handshake de specs no arranque. Drop fecha a thread. Teste de isolamento: a tool corre num ThreadId diferente do chamador; erro/limite de uma tool nao fecha o turno.
<!-- knudge:parent task_015epqsj -->