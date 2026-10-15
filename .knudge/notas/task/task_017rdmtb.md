---
id: task_017rdmtb
type: task
statement: Harness de testes: unit + property + goldens + provider falso com fixtures; CI mínimo
created_at: 2026-10-13T00:50:48.179Z
body_hash: 2b710385
schema_version: 2
tags: [task]
revision: 6
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:09:37.526Z
    notes: harness: unit (15) + property proptest (mem_fs_round_trips_arbitrary_content) + goldens (layout e JSON de metricas) + CI .github/workflows/ci.yml; o provider falso com fixtures e a task G1 task_00mabsbe
  - status: success
    recorded_at: 2026-10-15T09:26:57.418Z
    notes: harness: unit + property (crates/ktu-core/tests/ports_properties.rs::mem_fs_round_trips_arbitrary_content) + goldens de layout (layout.rs::init_dirs_match_the_golden) e de JSON de metricas (ports/metrics.rs::snapshot_serialises_to_golden_json) + CI .github/workflows/ci.yml
anchors: [crates/ktu-core/tests/ports_properties.rs, crates/ktu-core/src/layout.rs, .github/workflows/ci.yml]
status: closed
scope: task
---
<!-- knudge:parent task_00qm4eu6 -->