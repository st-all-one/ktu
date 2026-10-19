---
id: task_015a04hb
type: task
statement: Migracoes de config: versao de schema + migracao idempotente e leitura nao-destrutiva
created_at: 2026-10-17T23:51:50.999Z
body_hash: 0479df45
schema_version: 2
tags: [fase3]
revision: 5
outcomes:
  - status: success
    recorded_at: 2026-10-18T16:37:39.914Z
    notes: "F5: crates/ktu-core/src/config/migrate.rs (schema=1, schema_of, read_migrate nao destrutiva, migrate idempotente com lista de obsoletas vazia, fail-closed em versao futura) + crates/ktu-cli/src/config_doctor.rs (config doctor, read-only) + config_cmd (leitura migra em memoria; escrita carimba e avisa em stderr); 18 testes no core + 4 de integracao em crates/ktu-cli/tests/config.rs; gate G20 verde 7/7; ADR 0020."
anchors: [wiki/implementation_phase3/F5-migracoes-config.md]
status: closed
scope: task
---
<!-- knudge:parent task_00g0us7a -->