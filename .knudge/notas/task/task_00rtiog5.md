---
id: task_00rtiog5
type: task
statement: Contrato de maquina: params <JSON> (- le stdin) com schema versionado e precedencia
created_at: 2026-10-17T22:54:26.533Z
body_hash: d0cadaee
schema_version: 2
tags: [fase2]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-17T23:23:00.579Z
    notes: "params.rs (RunParams serde, deny_unknown_fields, pick flag>params>default); flags do Run passaram a Option; precedencia verificada; make check verde"
anchors: [crates/ktu-cli/src/params.rs, crates/ktu-cli/src/cli.rs, crates/ktu-cli/src/main.rs, wiki/backlog/cli-superficie.md]
status: closed
scope: task
---
<!-- knudge:parent task_00qn6s14 -->