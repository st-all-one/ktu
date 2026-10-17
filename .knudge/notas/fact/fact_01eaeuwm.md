---
id: fact_01eaeuwm
type: fact
statement: T2: tools correm numa thread dedicada (ktu-tools-t2) via mpsc+oneshot; I8 verificavel por teste
created_at: 2026-10-17T01:06:47.722Z
body_hash: 5dbd0cfc
schema_version: 2
anchors: [crates/ktu-tools/src/tool_thread.rs]
---
ktu-tools/src/tool_thread.rs: ToolThread::spawn cria a T2 que constroi o executor dentro da thread (WorkerCtx::serve) e serve pedidos por tokio mpsc unbounded + oneshot. O kernel (T1) fala so por mensagens. Handshake de specs no arranque; Drop larga o emissor e faz join. CLI usa ToolThread em vez do Toolbox directo. Teste tools_run_on_a_dedicated_thread prova ThreadId diferente; tool_errors_are_contained. Deadlock inicial (emissor vivo durante join) corrigido.