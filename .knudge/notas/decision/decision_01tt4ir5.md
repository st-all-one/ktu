---
id: decision_01tt4ir5
type: decision
statement: "ktu-memory usa o knudge-core como motor puro em memoria sobre notes.jsonl, sem projetar a porta Fs"
created_at: 2026-10-13T02:45:17.282Z
body_hash: 8ca76f4d
schema_version: 2
anchors: [crates/ktu-memory/tests/knudge_spike.rs]
---
Por quê: validado por spike que Index::build+recall e o codec JSON do knudge-core funcionam sobre Vec<Note> em memoria (sem Store/Fs). Alternativas descartadas: (a) projetar um Fs virtual que emule notas/*.md a partir do JSONL — complexo e acopla a convencao de caminhos privada do knudge; (b) reimplementar retrieval — reinventa o que a decisao_0166fqyj mandou adotar. Consequência: JSONL e a fonte da verdade; o indice (Index/Graph) e derivado e reconstruido a cada operacao; hints sao ponteiros (id+statement+score).