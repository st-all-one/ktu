---
id: task_01iz1xh1
type: task
statement: Memória knudge & persistência (sessão a sessão)
created_at: 2026-10-13T00:50:48.975Z
body_hash: 9021bbae
schema_version: 2
tags: [issue, g3]
results_in: [task_00uz2my4, task_00c0u0vr, task_00z0ju90, task_01jeruz5, task_01s1vlkb]
revision: 8
outcomes:
  - status: success
    recorded_at: 2026-10-13T03:02:43.420Z
    notes: "G3 (memoria knudge & persistencia) fechado. Entregue: (1) validacao do motor knudge-core em memoria + schema fisico JSONL; (2) porta Memory em ktu_core::memory + adaptador ktu-memory::JsonlMemory (append-only, dedup-on-read, escrita duravel, recall BM25+RRF, dedup 0.75/0.92, hints como ponteiros); (3) gate no loop (recall no inicio de sessao + tools memory_ask/memory_write); (4) compactacao aditiva + checkpoint tipado duravel (temp->fsync->rename) + recuperacao de crash (load_recovering); (5) ktu memory export/list + ktu doctor --memory. Evidencia: make check verde com 92 testes; smoke real (ktu run --provider mock) grava .ktu/sessions/s1.jsonl + s1.checkpoint.json. Persistencia entre sessoes provada por notes_persist_across_adapters; compacted:true por a_tight_budget_compacts_instead_of_failing. Proximo: G4 (gate inegociavel quality_test)."
anchors: [plan.md]
status: closed
scope: issue
---
<!-- knudge:parent epic_00ynu6q8 -->