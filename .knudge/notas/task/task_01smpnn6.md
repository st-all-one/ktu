---
id: task_01smpnn6
type: task
statement: Fila de mutacao de ficheiro (95) + truncagem sistematica (94/G3)
created_at: 2026-10-17T22:54:29.294Z
body_hash: 3ad7cc71
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: partial
    recorded_at: 2026-10-18T00:31:59.635Z
    notes: "Truncagem sistematica feita (truncate.rs: linhas+bytes, sem linhas parciais, spill para .ktu/out/ com ponteiro). Fila de mutacao por caminho adiada: as tools correm sequencialmente na T2 (sem concorrencia) -> A6."
anchors: [wiki/backlog/fila-mutacao-ficheiro.md]
status: closed
scope: task
---
<!-- knudge:parent task_00htbspv -->