---
id: task_00xgy3g9
type: task
statement: Precedencia (flag>params>local>global; credenciais flag>env>auth) + schema + migracoes
created_at: 2026-10-17T22:54:08.265Z
body_hash: 1e32f7a2
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: partial
    recorded_at: 2026-10-17T23:49:38.901Z
    notes: "Precedencia pura (Config::merge local>global) + schema fechado (deny_unknown_fields, KEYS estaveis) + integracao no run/serve (flag > params > local > global > default). Adiado (YAGNI): migracoes — schema ainda v1, sem versao anterior a migrar"
anchors: [wiki/backlog/config-precedencia.md]
status: closed
scope: task
---
<!-- knudge:parent task_00tx0uvu -->