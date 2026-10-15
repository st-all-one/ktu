---
id: task_0118dhru
type: task
statement: BudgetGate com dono único + fixar a instrução corrente (I14)
created_at: 2026-10-13T00:50:48.484Z
body_hash: 77e1682c
schema_version: 2
tags: [task]
revision: 6
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:16:27.331Z
    notes: coberto por ktu-core (domain/session/agent/guard/budget/testing) + tests/loop.rs; make check verde
  - status: success
    recorded_at: 2026-10-15T09:26:57.029Z
    notes: "ktu-core/src/budget.rs (BudgetGate, dono unico do tecto de contexto, I14) + compactacao em compact.rs; teste a_tight_budget_compacts_instead_of_failing"
anchors: [crates/ktu-core/src/budget.rs]
status: closed
scope: task
---
<!-- knudge:parent task_01xvduai -->