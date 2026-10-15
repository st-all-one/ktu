---
id: task_00mow2ux
type: task
statement: "Padrão de logs estruturados: stdout=dados, stderr=logs, ids de evento estáveis, custo zero por omissão"
created_at: 2026-10-13T00:50:48.098Z
body_hash: ef73b7ce
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:08:02.504Z
    notes: porta Logger (Level/LogRecord/Logger) em ktu-core::ports; ids estaveis em ktu_core::events; fakes Noop/Recording (12 testes); adaptador tracing JSON->stderr na borda (ktu-cli/src/logging.rs); make check verde
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_00qm4eu6 -->