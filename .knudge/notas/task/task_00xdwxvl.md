---
id: task_00xdwxvl
type: task
statement: Quebrar a god-struct Deps em Ports/Telemetry/Capabilities
created_at: 2026-10-17T00:42:24.439Z
body_hash: eebdae7c
schema_version: 2
tags: [deps]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T01:06:55.158Z
    notes: Deps agregado em Ports/Telemetry/Capabilities; 11 literais de teste + run.rs + subagents.rs actualizados; make check verde.
anchors: [crates/ktu-core/src/agent/mod.rs]
status: closed
scope: task
---
Deps passa a agregar três structs coesas: Ports (provider, tools, guard, budget, cancel), Telemetry (logger, metrics, clock) e Capabilities (memory, subagents). Actualizar todos os pontos de uso e os literais de teste. Objectivo: reduzir o acoplamento e preparar a G6 (a porta precisa de mais dependencias).
<!-- knudge:parent task_015epqsj -->