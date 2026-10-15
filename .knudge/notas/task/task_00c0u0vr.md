---
id: task_00c0u0vr
type: task
statement: "Porta Memory (tipos do ktu) + adaptador knudge-core; hints são ponteiros (id+statement+score), nunca o corpo"
created_at: 2026-10-13T00:50:49.081Z
body_hash: 6ad37422
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-13T02:50:45.092Z
    notes: "Porta Memory (tipos do ktu) + adaptador knudge-core implementados. ktu_core::memory define Memory (recall/write), Hint (ponteiro id+statement+score), MemoryDraft, MemoryWrite (created/merged/rejected/unchanged) e MemoryError. ktu-memory::JsonlMemory implementa a porta sobre .agents/memory/notes.jsonl: append-only com dedup-on-read, escrita duravel (temp->fsync->rename), recall via Index::build+Graph::from_notes_ref+recall do knudge-core, dedup via DeduceThresholds 0.75/0.92 + write::propose, merge de tags/ancoras/corpo com revisao+1. Testes (5 de integracao + 3 unit + 2 spike) verdes; clippy -D warnings limpo. Fs ganhou sync() (durabilidade)."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_01iz1xh1 -->