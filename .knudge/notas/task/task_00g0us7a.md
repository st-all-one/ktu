---
id: task_00g0us7a
type: task
statement: "F5 - Migracoes de config (schema versionado, leitura nao-destrutiva)"
created_at: 2026-10-18T13:31:56.452Z
body_hash: 5f2e3972
schema_version: 2
tags: [fase3]
results_in: [task_015a04hb]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-18T16:37:40.487Z
    notes: F5 fechada (gate G20 7/7). Sem o gatilho declarado (schema v2 com configs v1 em uso): o mecanismo de versionamento e pre-condicao de qualquer migracao futura; desvio registado na ADR 0020. O passo 2 do plano (retirar chaves planas antigas) ficou com a lista vazia - nao existe chave obsoleta - e o mecanismo e testado com um passo de ensaio.
anchors: [wiki/implementation_phase3/F5-migracoes-config.md]
status: closed
scope: issue
---
<!-- knudge:parent epic_00j1cgo3 -->