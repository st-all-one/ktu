---
id: task_00sv9qlq
type: task
statement: "Agente moderno real: provider llama.cpp, tools de filesystem, sessão e cache"
created_at: 2026-10-13T00:50:48.625Z
body_hash: 2a957b63
schema_version: 2
tags: [issue, g2]
results_in: [task_016rvfop, task_01j2o6cg, task_00si6z5t, task_01bzrwbx, task_01ptulp8, task_01ma94kf, task_01chmvy6, task_018qh88b, task_01xdbyx0]
revision: 12
outcomes:
  - status: success
    recorded_at: 2026-10-13T02:39:50.393Z
    notes: "G2 (agente moderno real) fechado: provider llama.cpp (SSE streaming, tool-calling nativo + fallback), test double HTTP, tools fs (read/write/edit/list/glob/grep) + bash + trash, sessao resume + projecao de wire determinista, ktu providers doctor, e modelo Ornith-1.5-9B-Q4_K_M a correr em llama.cpp stock com sonda de tool-calling verde. make check verde."
anchors: [plan.md]
status: closed
scope: issue
---
<!-- knudge:parent epic_00ynu6q8 -->