---
id: fact_00lt7c6s
type: fact
statement: "tool.result emitido num ponto unico (push_result): cobre negacao, cap e sintetico; risco Logger resolvido"
created_at: 2026-10-17T01:24:03.658Z
body_hash: fa88aa72
schema_version: 2
anchors: [crates/ktu-core/src/agent/mod.rs]
---
ktu-core/src/agent/mod.rs: Driver::push_result emite support::log_tool_result antes de empurrar Event::ToolResult, cobrindo executado/negado/escalado/cap/sintetico. support::log_guard_decision (em evaluate_guard) e log_tool_call/log_tool_result (execute_effect, delegate) tambem vao pela porta Logger. Provado por guard_denies_a_tool_outside_the_spec e repeated_calls_are_detected_as_stagnation.