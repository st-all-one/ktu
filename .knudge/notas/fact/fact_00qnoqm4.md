---
id: fact_00qnoqm4
type: fact
statement: O knudge-core nao indexa notas JSONL: enumera notas/<tipo>/<id>.md via Fs; JSONL so serve eventos e indice derivado
created_at: 2026-10-13T02:45:23.987Z
body_hash: afe08289
schema_version: 2
anchors: [crates/ktu-memory/Cargo.toml]
---
Evidência: REF/knudge/crates/knudge-core/src/store/mod.rs (note_path = notas/<tipo>/<id>.md; list_ids filtra .md) e jsonl/mod.rs (usado por eventos/.idx). Consequência: para manter JSONL como container, o ktu-memory materializa as notas em memória e usa o motor puro do knudge-core, sem projecao de Fs.