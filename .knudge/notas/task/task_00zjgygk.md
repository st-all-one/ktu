---
id: task_00zjgygk
type: task
statement: AgentSpec + registo de agentes (primário ktu + explorer read-only estrutural)
created_at: 2026-10-13T00:50:49.650Z
body_hash: ce41d5a8
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T00:14:39.883Z
    notes: "AgentSpec (id + conjunto de tools + max_depth) e AgentRegistry (primario ktu + explorer read-only) em ktu-core/src/agent/spec.rs; default_registry em ktu-tools/src/agents.rs (explorer sem write/edit/bash/trash/memory_write, max_depth 0); StructuralGuard::for_spec deriva do AgentSpec; CLI usa o registo. 6 testes novos; make check verde."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_00j43pma -->