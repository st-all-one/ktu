---
id: task_01r72uu1
type: task
statement: Ctrl-C cancela o turno sem matar o processo + Tab (raw mode gated por TTY)
created_at: 2026-10-18T13:32:17.666Z
body_hash: f944b621
schema_version: 2
tags: [fase3]
revision: 3
outcomes:
  - status: partial
    recorded_at: 2026-10-18T14:25:04.748Z
    notes: "Ctrl-C feito (gate G17 checks 8-10): tarefa de borda com tokio::signal::ctrl_c() -> hub.cancel(id); o processo nao morre e o turno fecha com StopReason::Cancelled (provado com um plugin observe lento + kill -INT). A Tab fica ADIADA: exige raw mode (crossterm) e gestao do terminal em volta dos logs do turno (o REPL escreve em stderr durante o turno) - nao e um if, e um editor de linha. Tarefa propria criada: task_002em5xu."
anchors: [wiki/implementation_phase3/F2-repl-interactivo.md]
status: closed
scope: task
---
<!-- knudge:parent task_01mrz94g -->