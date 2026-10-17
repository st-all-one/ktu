---
id: task_01sai4yj
type: task
statement: Fan-out: N subagentes em paralelo com join e caps; ordem de resultados determinística no log
created_at: 2026-10-13T00:50:49.792Z
body_hash: 8264a120
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T00:22:54.368Z
    notes: "Fan-out em agent/delegation.rs: run_calls avalia guardrails, corre subagentes do mesmo passo com join_all (paralelo) e as restantes tools sequencialmente; resultados no log pela ordem original (determinismo). Cap LoopConfig.max_subagents_per_turn (default 4). Testes fan_out_runs_subagents_and_keeps_result_order e fan_out_respects_the_per_turn_cap; make check verde."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_00j43pma -->