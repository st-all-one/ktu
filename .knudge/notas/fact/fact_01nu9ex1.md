---
id: fact_01nu9ex1
type: fact
statement: O checkpoint da sessao e tipado e duravel (temp->fsync->rename) e reproduz o corte de compactacao ao retomar
created_at: 2026-10-13T03:02:29.114Z
body_hash: "51e91418"
schema_version: 2
anchors: [crates/ktu-core/src/checkpoint.rs]
---
Evidência: crates/ktu-core/src/checkpoint.rs (Checkpoint com session/turns/compacted/keep_from/updated_ms) e store::load_recovering (tolera a ultima linha parcial de um crash). run.rs grava o checkpoint apos cada turno e aplica keep_from ao retomar. Fs ganhou sync() para o fsync.