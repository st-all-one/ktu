---
id: fact_01oujbqw
type: fact
statement: G1 revisado e confirmado: fecha os 7 criterios da prova de vida com make check verde (40 testes)
created_at: 2026-10-13T01:40:13.738Z
body_hash: ac38fb98
schema_version: 2
anchors: [plan.md, crates/ktu-core/src/agent/mod.rs]
---
Evidência: make check (fmt + clippy -D warnings + test + xtask check-layers + file-length 300) verde; 1+29+5+1+4 = 40 testes. Confirmado por leitura: log JSONL é fonte da verdade e derive_messages projeta 1 assistant com N tool_calls + N results (session.rs); loop cancelável com StopReason obrigatório, estagnação e MaxSteps (agent/mod.rs); guard estrutural capability/path/depth/step + envelope taint (guard.rs, taint.rs); BudgetGate dono único (budget.rs); provider/tools com fakes determinísticos (testing.rs); read_file com StdFs + envelope taint; ktu run embedded com stdout=dados e stderr=logs JSON.