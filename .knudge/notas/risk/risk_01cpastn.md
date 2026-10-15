---
id: risk_01cpastn
type: risk
statement: O loop so emite turn.start/turn.end/kernel.error pela porta Logger (contrato §1.3 parcial)
created_at: 2026-10-13T01:40:18.395Z
body_hash: 482a05a0
schema_version: 2
anchors: [crates/ktu-core/src/agent/support.rs]
---
Evidência: agent/support.rs so chama log_event com TURN_START/TURN_END e KERNEL_ERROR; guard.decision e tool.result ficam apenas no SessionLog, nao no stderr estruturado. Consequência: ligar model.request/model.output, tool.call/tool.result e guard.decision ao Logger quando o provider real e as tools forem cablados no G2.