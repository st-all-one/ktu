---
id: task_01bzrwbx
type: task
statement: bash com timeout/cap + análise de argv + retry/backoff do provider
created_at: 2026-10-13T00:50:48.818Z
body_hash: 96bd62d7
schema_version: 2
tags: [task]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:55:54.211Z
    notes: "BashTool (ktu-tools/src/bash.rs): executa 'bash -c' no root com timeout (kill_on_drop mata o processo), cap de saida (32 KiB) e analise de argv fail-closed que recusa padroes destrutivos (rm -rf/mkfs/sudo/fork bomb/dd). Toolbox (toolbox.rs) agrega FsTools+bash num unico ToolExecutor. Retry/backoff do provider ja implementado em llama.rs (5xx/indisponivel). 4 testes de integracao (bash.rs) + 1 unit; make check verde (64 testes)."
anchors: [crates/ktu-tools/src/bash.rs, crates/ktu-tools/src/toolbox.rs]
status: closed
scope: task
---
<!-- knudge:parent task_00sv9qlq -->