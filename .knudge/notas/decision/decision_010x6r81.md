---
id: decision_010x6r81
type: decision
statement: O loop do ktu deriva todo o pedido do log da sessao (I3) e cancela ao nivel de passo nesta fase
created_at: 2026-10-13T01:18:17.386Z
body_hash: 0b292b2b
schema_version: 2
revision: 2
anchors: [crates/ktu-core/src/agent/mod.rs]
---
Por quê: derivar do log dá replay/resume/compactação/testes de graça; em F1 não há streaming real. Evidência: ktu-core/src/agent + tests/loop.rs (EndTurn/MaxSteps/Stalled/Cancelled/Error). Consequência: cancelamento a meio do stream chega com o provider real (F2).