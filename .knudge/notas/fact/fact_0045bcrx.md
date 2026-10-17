---
id: fact_0045bcrx
type: fact
statement: Fan-out: subagents do mesmo passo correm com join_all e resultados entram por ordem de chamada
created_at: 2026-10-17T00:22:58.013Z
body_hash: db8af10d
schema_version: 2
anchors: [crates/ktu-core/src/agent/delegation.rs]
---
agent/delegation.rs::Driver::run_calls separa subagentes (Pending::Delegate) das tools normais (Pending::Effect): avalia guardrails primeiro, corre os subagentes com futures::future::join_all e as restantes tools sequencialmente, e empurra os ToolResult na ordem original. Cap por turno em LoopConfig.max_subagents_per_turn (default 4). make check verde.