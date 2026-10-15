---
id: task_01ma94kf
type: task
statement: "Trash em vez de apagar (recuperável, com índice) + ktu providers doctor"
created_at: 2026-10-13T00:50:48.925Z
body_hash: 82fa65d6
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-13T02:04:24.436Z
    notes: "Trash recuperavel: ktu-tools/src/trash.rs (TrashTool 'trash_file' move para .ktu/trash/<id>, indice .ktu/trash/index.jsonl; list/restore reconstroem). CLI: ktu trash list|restore <id>; run cria .ktu/trash. 1 teste de integracao (mover -> listar -> restaurar). ktu providers doctor implementado (ktu-provider/src/doctor.rs + CLI providers.rs): verifica /v1/models, /props (model_path, n_ctx), GGUF (magic+metadados via gguf.rs), aviso de pack ternary e sonda de tool-calling opcional; verificado e2e (sem servidor -> inacessivel; com GGUF sintetico -> architecture=bonsai, file_type=999, aviso fork)."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_00sv9qlq -->